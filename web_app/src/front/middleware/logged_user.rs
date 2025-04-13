use ntex::{
    http::Payload,
    web::{Error, FromRequest, HttpRequest},
};
use ntex_identity::RequestIdentity;

use crate::front::errors;
use crate::models;

pub struct CheckUserCanAccessService;

pub struct IsUserLoggedAndCanEdit(pub bool, pub Option<i64>);

fn serialize_logged_user(str: &str) -> serde_json::Result<models::user_app::User> {
    serde_json::from_str::<models::user_app::User>(str)
}

/// Extracts the [LoggedUser] from a string session cookie
fn get_logged_user(auth_cookie: Option<String>) -> Result<models::user_app::User, Error> {
    serialize_logged_user(&auth_cookie.unwrap_or_default())
        .map_err(|_| errors::UserError::Unauthorized.into())
}

impl<Err> FromRequest<Err> for models::user_app::User {
    type Error = Error;

    fn from_request(
        req: &HttpRequest,
        _: &mut Payload,
    ) -> impl std::future::Future<Output = Result<Self, Self::Error>> {
        let identity_cookie = req.get_identity();
        futures::future::ready(get_logged_user(identity_cookie))
    }
}

impl<Err> FromRequest<Err> for CheckUserCanAccessService {
    type Error = Error;

    fn from_request(
        req: &HttpRequest,
        _: &mut Payload,
    ) -> impl std::future::Future<Output = Result<Self, Self::Error>> {
        let identity_cookie = req.get_identity();
        match get_logged_user(identity_cookie) {
            Ok(user) => futures::future::ready(if user.can_access_service() {
                Ok(Self)
            } else {
                Err(errors::UserError::NeedSubscription.into())
            }),
            Err(err) => futures::future::ready(Err(err)),
        }
    }
}

fn check_can_edit(auth_cookie: Option<String>) -> IsUserLoggedAndCanEdit {
    get_logged_user(auth_cookie)
        .map(|user| IsUserLoggedAndCanEdit(user.can_access_service(), Some(user.id)))
        .unwrap_or(IsUserLoggedAndCanEdit(false, None))
}

impl<Err> FromRequest<Err> for IsUserLoggedAndCanEdit {
    type Error = Error;

    fn from_request(
        req: &HttpRequest,
        _: &mut Payload,
    ) -> impl std::future::Future<Output = Result<Self, Self::Error>> {
        let identity_cookie = req.get_identity();

        futures::future::ready(Ok(check_can_edit(identity_cookie)))
    }
}
