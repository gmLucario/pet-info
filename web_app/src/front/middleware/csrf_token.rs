use base64::{Engine, prelude::BASE64_STANDARD};
use csrf::CsrfProtection;
use ntex::{http::Payload, web};
use ntex_session::UserSession;

use crate::{
    consts,
    front::{AppState, errors},
};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct CsrfToken {
    pub token_base64: String,
    pub cookie_base64: String,
}

fn is_csrf_valid(req: &web::HttpRequest) -> bool {
    if let (Ok(Some(csrf)), Some(app_state)) = (
        req.get_session()
            .get::<String>(consts::CSRF_TOKEN_COOKIE_NAME),
        req.app_state::<AppState>(),
    ) {
        let csrf = serde_json::from_str::<CsrfToken>(&csrf).unwrap_or_default();
        let token = BASE64_STANDARD
            .decode(csrf.token_base64.as_bytes())
            .map(|token| app_state.csrf_protec.parse_token(&token));
        let cookie = BASE64_STANDARD
            .decode(csrf.cookie_base64.as_bytes())
            .map(|cookie| app_state.csrf_protec.parse_cookie(&cookie));

        if let (Ok(Ok(token)), Ok(Ok(cookie))) = (token, cookie) {
            return app_state
                .csrf_protec
                .verify_token_pair(&token, &cookie)
                .is_ok();
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
        if !is_csrf_valid(req) {
            return std::future::ready(Err(errors::ServerError::InvalidCsrfToken.into()));
        }

        std::future::ready(Ok(Self::default()))
    }
}
