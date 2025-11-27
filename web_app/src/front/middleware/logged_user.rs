use ntex::{
    http::Payload,
    web::{Error, FromRequest, HttpRequest},
};
use ntex_identity::RequestIdentity;

use crate::front;

/// Extract logged user session from request
impl<Err> FromRequest<Err> for front::session::WebAppSession {
    type Error = Error;

    fn from_request(
        req: &HttpRequest,
        _: &mut Payload,
    ) -> impl std::future::Future<Output = Result<Self, Self::Error>> {
        let identity_cookie = req.get_identity();
        futures::future::ready(get_logged_user_session(identity_cookie))
    }
}

/// Check user has valid service membership
pub struct CheckUserCanAccessService;

/// Check user is logged and can edit
pub struct IsUserLoggedAndCanEdit(pub bool, pub Option<i64>);

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

impl<Err> FromRequest<Err> for CheckUserCanAccessService {
    type Error = Error;

    fn from_request(
        req: &HttpRequest,
        _: &mut Payload,
    ) -> impl std::future::Future<Output = Result<Self, Self::Error>> {
        let identity_cookie = req.get_identity();
        match get_logged_user_session(identity_cookie) {
            Ok(session) => futures::future::ready(if session.user.can_access_service() {
                Ok(Self)
            } else {
                Err(front::errors::UserError::NeedSubscription.into())
            }),
            Err(err) => futures::future::ready(Err(err)),
        }
    }
}

fn serialize_logged_user_session(str: &str) -> serde_json::Result<front::session::WebAppSession> {
    serde_json::from_str::<front::session::WebAppSession>(str)
}

/// Extract session from auth cookie
fn get_logged_user_session(
    auth_cookie: Option<String>,
) -> Result<front::session::WebAppSession, Error> {
    serialize_logged_user_session(&auth_cookie.unwrap_or_default())
        .map_err(|_| front::errors::UserError::Unauthorized.into())
}

fn check_can_edit(auth_cookie: Option<String>) -> IsUserLoggedAndCanEdit {
    get_logged_user_session(auth_cookie)
        .map(|session| {
            IsUserLoggedAndCanEdit(session.user.can_access_service(), Some(session.user.id))
        })
        .unwrap_or(IsUserLoggedAndCanEdit(false, None))
}
