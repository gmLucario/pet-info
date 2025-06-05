use csrf::CsrfProtection;
use ntex::web;
use ntex_identity::Identity;
use oauth2::{AuthorizationCode, CsrfToken, TokenResponse, reqwest};
use serde::Deserialize;

use crate::{
    api, consts,
    front::{AppState, errors, middleware, oauth, session, utils},
};

/// Google oauth minimum data to handle the login callback request
#[derive(Deserialize, Debug)]
struct Q {
    code: String,
    state: String,
    // scope: String,
}

/// Google oauth minimum data to handle the login callback request
#[derive(Deserialize, Clone, Debug)]
pub struct UserProfile {
    email: String,
}

/// Endpoint handles the google oauth callback login from index login
#[web::get("/google_callback")]
async fn google_callback(
    q: web::types::Query<Q>,
    cookie: ntex_session::Session,
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
                        "at google oauth token creation: {e}"
                    ))
                })?,
        )
        .await
        .map_err(|e| {
            errors::ServerError::ExternalServiceError(format!(
                "at google oauth token creation: {e}"
            ))
        })?
        .access_token()
        .secret()
        .to_string();

    let profile = crate::utils::REQUEST_CLIENT
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

    let (csrf_token, csrf_cookie) = app_state
        .csrf_protec
        .generate_token_pair(None, consts::MAX_AGE_COOKIES)
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!("cant set token csrf protection: {e}"))
        })?;

    cookie.set(
        consts::CSRF_TOKEN_COOKIE_NAME,
        serde_json::to_string(&middleware::csrf_token::CsrfToken {
            token_base64: csrf_token.b64_string(),
            cookie_base64: csrf_cookie.b64_string(),
        })?,
    )?;

    let user = api::user::get_or_create_app_user_by_email(&app_state.repo, &profile.email)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at /google_callback user could not be retrieved: {e}"
            ))
        })?;

    let is_user_enabled = user.is_enabled;
    let user_id = user.id;

    identity.remember(serde_json::to_string(&session::WebAppSession {
        user,
        add_pet_balance: api::user::get_user_add_pet_balance(&app_state.repo, user_id)
            .await
            .map_err(|e| {
                errors::ServerError::InternalServerError(format!(
                    "cant get user add pet balance {e}"
                ))
            })?,
    })?);

    if !is_user_enabled {
        return utils::redirect_to("/reactivate-account");
    }

    if let Ok(Some(redirect_to)) = cookie.get::<String>(consts::REDIRECT_TO_COOKIE_NAME) {
        cookie.remove(consts::REDIRECT_TO_COOKIE_NAME);
        return utils::redirect_to(&redirect_to);
    }

    utils::redirect_to("/pet")
}
