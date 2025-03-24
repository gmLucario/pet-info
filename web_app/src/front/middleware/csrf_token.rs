use base64::{prelude::BASE64_STANDARD, Engine};
use csrf::CsrfProtection;
use ntex::{
    http::{HttpMessage, Payload},
    service::{Middleware, Service, ServiceCtx},
    web,
};
use ntex_session::UserSession;

use crate::{
    front::{errors, AppState},
    {config, consts},
};

#[derive(Default)]
pub struct CsrfToken;

pub struct CsrfTokenMiddleware<S> {
    service: S,
}

impl<S> Middleware<S> for CsrfToken {
    type Service = CsrfTokenMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        CsrfTokenMiddleware { service }
    }
}

impl<S, Err> Service<web::WebRequest<Err>> for CsrfTokenMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
    Err: web::ErrorRenderer,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        req: web::WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        if req.cookie("ntex-session").is_some() {
            return ctx.call(&self.service, req).await;
        }

        if let Some(app_state) = req.app_state::<AppState>() {
            if let Ok((token, _)) = app_state.csrf_protec.generate_token_pair(None, 300) {
                req.get_session()
                    .set(consts::CSRF_TOKEN_COOKIE_NAME, token.b64_string())?;
            }
        }

        ctx.call(&self.service, req).await
    }
}

fn is_csrf_valid(req: &web::HttpRequest) -> bool {
    if let (Ok(Some(token)), Some(app_state)) = (
        req.get_session()
            .get::<String>(consts::CSRF_TOKEN_COOKIE_NAME),
        req.app_state::<AppState>(),
    ) {
        if let Ok(token_bytes) = BASE64_STANDARD.decode(token.as_bytes()) {
            return app_state.csrf_protec.parse_token(&token_bytes).is_ok();
        }
    }

    false
}

impl<Err> web::FromRequest<Err> for CsrfToken {
    type Error = web::Error;

    fn from_request(
        req: &web::HttpRequest,
        _: &mut Payload,
    ) -> impl std::future::Future<Output = Result<Self, Self::Error>> {
        if config::APP_CONFIG.is_prod() && !is_csrf_valid(req) {
            return std::future::ready(Err(errors::ServerError::InvalidCsrfToken.into()));
        }

        std::future::ready(Ok(Self {}))
    }
}
