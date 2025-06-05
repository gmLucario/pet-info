//! Handlers not linked to a specific url

use ntex::web;
use ntex_files::NamedFile;
use ntex_identity::Identity;
use serde_json::json;

use crate::{
    api, consts,
    front::{AppState, errors, middleware, oauth, session, templates, utils},
};

/// Serve `favicon.ico`
#[web::get("/favicon.ico")]
async fn serve_favicon() -> Result<impl web::Responder, web::Error> {
    Ok(NamedFile::open("web/static/images/favicon.ico")?)
}

/// Return a [UrlNotFound](errors::UserError::UrlNotFound) error for urls not defined
pub async fn serve_not_found() -> Result<web::HttpResponse, web::Error> {
    Err(errors::UserError::UrlNotFound.into())
}

/// Endpoint to render the index view
#[web::get("/")]
async fn index(cookie: ntex_session::Session) -> Result<impl web::Responder, web::Error> {
    let (auth_url, csrf_state) = oauth::get_new_auth_url();

    cookie
        .set(consts::CSRF_STATE_COOKIE_NAME, csrf_state)
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at index.html cant set CSRF_STATE_COOKIE_NAME: {e}"
            ))
        })?;

    let context = tera::Context::from_value(json!({
        "google_outh_auth_url": &auth_url,
        "service_price": &format!("{:.2}", consts::ADD_PET_PRICE),
    }))
    .unwrap_or_default();

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            templates::WEB_TEMPLATES
                .render("index.html", &context)
                .map_err(|e| {
                    errors::ServerError::TemplateError(format!(
                        "at /index endpoint the template couldnt be rendered: {e}"
                    ))
                })?,
        ))
}

/// Endpoint to render the reactivate account starting point
#[web::get("/reactivate-account")]
async fn get_reactivate_account_view(
    _: session::WebAppSession,
) -> Result<impl web::Responder, web::Error> {
    Ok(web::HttpResponse::Ok().body(
        templates::WEB_TEMPLATES
            .render("reactivate_account.html", &tera::Context::new())
            .map_err(|e| {
                errors::ServerError::TemplateError(format!(
                    "at /reactivate-account endpoint the template couldnt be rendered: {e}"
                ))
            })?,
    ))
}

/// Endpoint handles the request to reactivate an account.
/// An account is deactivated if the user deleted all the data
#[web::post("/reactivate-account")]
async fn reactivate_account(
    _: middleware::csrf_token::CsrfToken,
    mut user_session: session::WebAppSession,
    app_state: web::types::State<AppState>,
    identity: Identity,
    cookie: ntex_session::Session,
) -> Result<impl web::Responder, web::Error> {
    let user_id = user_session.user.id;
    api::user::reactivate_account(user_id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!("account couldnt be activated: {e}"))
        })?;

    user_session.user.is_enabled = true;

    //unwrap cause its safe, it comes internally
    identity.remember(serde_json::to_string(&user_session).unwrap());

    if let Ok(Some(redirect_to)) = cookie.get::<String>(consts::REDIRECT_TO_COOKIE_NAME) {
        cookie.remove(consts::REDIRECT_TO_COOKIE_NAME);
        return utils::redirect_to(&redirect_to);
    }

    utils::redirect_to("/pet")
}
