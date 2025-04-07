use oauth2::{
    AuthUrl, Client, ClientId, ClientSecret, RedirectUrl, RevocationUrl, Scope,
    StandardErrorResponse, TokenUrl, basic::BasicClient,
};
use std::sync::LazyLock;

use crate::{config::APP_CONFIG, consts};

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

pub fn get_google_outh_scopes() -> Vec<Scope> {
    vec![Scope::new(
        "https://www.googleapis.com/auth/userinfo.email".into(),
    )]
}

fn build_google_oauth_client() -> anyhow::Result<GoogleOauthClient> {
    Ok(
        BasicClient::new(ClientId::new(APP_CONFIG.google_oauth_client_id.to_string()))
            .set_client_secret(ClientSecret::new(
                APP_CONFIG.google_oauth_client_secret.to_string(),
            ))
            .set_auth_uri(AuthUrl::new(APP_CONFIG.google_oauth_auth_uri.to_string())?)
            .set_token_uri(TokenUrl::new(
                APP_CONFIG.google_oauth_token_uri.to_string(),
            )?)
            .set_redirect_uri(RedirectUrl::new(format!(
                "{}://{}/google_callback",
                APP_CONFIG.wep_server_protocol(),
                APP_CONFIG.url_host()
            ))?)
            .set_revocation_url(RevocationUrl::new(
                consts::GOOGLE_ENDPOINT_REVOKE_TOKEN.to_string(),
            )?),
    )
}

pub static GOOGLE_OAUTH: LazyLock<GoogleOauthClient> =
    LazyLock::new(|| build_google_oauth_client().unwrap());
