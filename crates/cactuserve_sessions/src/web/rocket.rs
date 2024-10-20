use ::rocket::{
    async_trait,
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use rocket::http::Cookie;

use crate::TokenReader;

use super::{from_str, Auth, AuthRequestError, KeyProvider, AUTH_HEADER_NAME, AUTH_HEADER_SCHEME};

#[async_trait]
impl<'r, R, K> FromRequest<'r> for Auth<R, K>
where
    R: TokenReader + Default,
    R::Error: std::fmt::Debug,
    K: KeyProvider,
{
    type Error = AuthRequestError<R::Error>;

    async fn from_request<'a>(request: &'r Request<'a>) -> Outcome<Self, Self::Error> {
        let Some(token) = request
            .headers()
            .get_one(AUTH_HEADER_NAME)
            .map(|header_value| {
                header_value
                    .strip_prefix(AUTH_HEADER_SCHEME)
                    .ok_or(AuthRequestError::<R::Error>::InvalidHeaderFormat)
                    .map(str::trim_start)
            })
            .or_else(|| {
                request
                    .cookies()
                    .get("cactuserve-session")
                    .map(Cookie::value)
                    .map(Ok)
            })
        else {
            return Outcome::Error((Status::Unauthorized, AuthRequestError::NoSession));
        };

        token
            .and_then(|token| {
                from_str::<R, _>(&R::default(), token, &K::public_key())
                    .map_err(AuthRequestError::BadToken)
            })
            .map_or_else(
                |err| Outcome::Error((Status::BadRequest, err)),
                |token| Outcome::Success(Self::new(token)),
            )
    }
}
