use ntex::web;
use ntex_files::NamedFile;
use ntex_identity::Identity;
use ntex_session::Session;
use oauth2::{reqwest, AuthorizationCode, CsrfToken, ResponseType, TokenResponse};
use serde::Deserialize;
use serde_json::json;

use crate::{
    api, consts,
    front::{errors, oauth, templates, utils, AppState},
    models,
};

#[web::get("/favicon.ico")]
async fn serve_favicon() -> Result<impl web::Responder, web::Error> {
    Ok(NamedFile::open("web/static/images/favicon.ico")?)
}

#[web::get("/rive/dog_hi.riv")]
async fn serve_dog_hi() -> Result<impl web::Responder, web::Error> {
    Ok(NamedFile::open("web/static/images/dog_hi.riv")?)
}

pub async fn serve_not_found() -> Result<web::HttpResponse, web::Error> {
    Err(errors::UserError::UrlNotFound.into())
}

#[web::get("/")]
async fn index(cookie: Session) -> Result<impl web::Responder, web::Error> {
    let (auth_url, csrf_state) = oauth::GOOGLE_OAUTH
        .authorize_url(CsrfToken::new_random)
        .add_scopes(oauth::get_google_outh_scopes())
        .set_response_type(&ResponseType::new("code".into()))
        .url();

    cookie
        .set(consts::CSRF_STATE_COOKIE_NAME, csrf_state)
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at index.html cant set CSRF_STATE_COOKIE_NAME: {}",
                e
            ))
        })?;

    let context = tera::Context::from_value(json!({
        "google_outh_auth_url": &auth_url,
        "service_price": &format!("{:.2}", consts::SERVICE_PRICE),
    }))
    .unwrap_or_default();

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            templates::WEB_TEMPLATES
                .render("index.html", &context)
                .map_err(|e| {
                    errors::ServerError::TemplateError(format!(
                        "at /index endpoint the template couldnt be rendered: {}",
                        e
                    ))
                })?,
        ))
}

#[web::get("/reactivate-account")]
async fn get_reactivate_account_view(
    _: models::user_app::User,
) -> Result<impl web::Responder, web::Error> {
    Ok(web::HttpResponse::Ok().body(
        templates::WEB_TEMPLATES
            .render("reactivate_account.html", &tera::Context::new())
            .map_err(|e| {
                errors::ServerError::TemplateError(format!(
                    "at /reactivate-account endpoint the template couldnt be rendered: {}",
                    e
                ))
            })?,
    ))
}

#[web::post("/reactivate-account")]
async fn reactivate_account(
    mut logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
    identity: Identity,
) -> Result<impl web::Responder, web::Error> {
    api::user::reactivate_account(logged_user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!("account couldnt be activated: {e}"))
        })?;

    logged_user.is_enabled = true;

    //unwrap cause its safe, it comes internally
    identity.remember(serde_json::to_string(&logged_user).unwrap());

    utils::redirect_to("/pet")
}

#[derive(Deserialize, Debug)]
struct Q {
    code: String,
    state: String,
    // scope: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserProfile {
    email: String,
}

#[web::get("/google_callback")]
async fn google_callback(
    q: web::types::Query<Q>,
    cookie: Session,
    app_state: web::types::State<AppState>,
    identity: Identity,
) -> Result<impl web::Responder, web::Error> {
    if q.state.ne(cookie
        .get::<CsrfToken>(consts::CSRF_STATE_COOKIE_NAME)?
        .unwrap_or(CsrfToken::new_random())
        .secret())
    {
        cookie.clear();
        return Err(errors::ServerError::InternalServerError(
            "at google_callback cant get CSRF_STATE_COOKIE_NAME".into(),
        )
        .into());
    }

    let token = oauth::GOOGLE_OAUTH
        .exchange_code(AuthorizationCode::new(q.code.to_string()))
        .request_async(
            &reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::limited(1))
                .build()
                .map_err(|e| {
                    errors::ServerError::ExternalServiceError(format!(
                        "at google oauth token creation: {}",
                        e
                    ))
                })?,
        )
        .await
        .map_err(|e| {
            errors::ServerError::ExternalServiceError(format!(
                "at google oauth token creation: {}",
                e
            ))
        })?
        .access_token()
        .secret()
        .to_string();

    let profile = crate::utils::request_client
        .get(consts::GOOGLE_ENDPOINT_USER_INFO)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            errors::ServerError::ExternalServiceError(format!("at get google user info: {}", e))
        })?
        .json::<UserProfile>()
        .await
        .map_err(|e| {
            errors::ServerError::ExternalServiceError(format!("at get google user info: {}", e))
        })?;

    let user = api::user::get_or_create_app_user_by_email(&app_state.repo, &profile.email)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at /google_callback user could not be retrieved: {}",
                e
            ))
        })?;

    //unwrap cause its safe, it comes internally
    identity.remember(serde_json::to_string(&user).unwrap());

    if !user.is_enabled {
        return utils::redirect_to("/reactivate-account");
    }

    if user.can_access_service() {
        return utils::redirect_to("/pet");
    }

    utils::redirect_to("/profile")
}
