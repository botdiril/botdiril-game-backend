use actix::fut::{err, ok, Ready};
use actix_web::dev::Payload;
use actix_web::error::ErrorInternalServerError;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};
use mongodb::bson::Bson;

pub mod jwt;

#[derive(Clone, Copy)]
pub struct User {
    pub key_ref_id: i64,
    pub id: UserId,
}

impl FromRequest for User {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        req.extensions().get::<User>().map_or_else(
            || err(ErrorInternalServerError("Internal server error.")),
            |u| ok(*u),
        )
    }
}

#[derive(
    Clone,
    Copy,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    derive_more::From,
    derive_more::Into,
)]
#[repr(transparent)]
pub struct UserId(i64);

impl From<UserId> for Bson {
    fn from(val: UserId) -> Self {
        val.0.into()
    }
}
