use oauth2::{
    AuthUrl, Client, ClientId, ClientSecret, CsrfToken, RedirectUrl, ResponseType, RevocationUrl,
    Scope, StandardErrorResponse, TokenUrl, basic::BasicClient,
};
use std::sync::LazyLock;

use crate::{config, consts};
use anyhow::Context;

pub type GoogleOauthClient = Client<
    StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
    oauth2::EndpointSet,
>;

pub static GOOGLE_OAUTH: LazyLock<GoogleOauthClient> =
    LazyLock::new(|| build_google_oauth_client().unwrap());

pub fn get_new_auth_url() -> (oauth2::url::Url, oauth2::CsrfToken) {
    GOOGLE_OAUTH
        .authorize_url(CsrfToken::new_random)
        .add_scopes(get_google_outh_scopes())
        .set_response_type(&ResponseType::new("code".into()))
        .url()
}

pub fn get_google_outh_scopes() -> Vec<Scope> {
    vec![Scope::new(
        "https://www.googleapis.com/auth/userinfo.email".into(),
    )]
}

fn build_redirect_url(redirect_url: &str) -> anyhow::Result<String> {
    Ok(format!(
        "{base_path}/{redirect_url}",
        base_path = config::APP_CONFIG
            .get()
            .context("failed to get app config")?
            .base_url(),
        redirect_url = redirect_url,
    ))
}

fn build_google_oauth_client() -> anyhow::Result<GoogleOauthClient> {
    let app_config = config::APP_CONFIG
        .get()
        .context("failed to get app config")?;
    Ok(
        BasicClient::new(ClientId::new(app_config.google_oauth_client_id.to_string()))
            .set_client_secret(ClientSecret::new(
                app_config.google_oauth_client_secret.to_string(),
            ))
            .set_auth_uri(AuthUrl::new(app_config.google_oauth_auth_uri.to_string())?)
            .set_token_uri(TokenUrl::new(
                app_config.google_oauth_token_uri.to_string(),
            )?)
            .set_redirect_uri(RedirectUrl::new(build_redirect_url("google_callback")?)?)
            .set_revocation_url(RevocationUrl::new(
                consts::GOOGLE_ENDPOINT_REVOKE_TOKEN.to_string(),
            )?),
    )
}
