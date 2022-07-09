use actix_web::dev::ServiceRequest;
use actix_web::error::{Error, ErrorBadRequest, ErrorInternalServerError};
use actix_web::web::Data;
use actix_web::HttpMessage;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use log::error;
use redis::{Client, Commands};
use serde::{Deserialize, Serialize};

use crate::auth::User;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    grant_type: String,
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let token = credentials.token();
    let header = jsonwebtoken::decode_header(token)
        .map_err(|_| ErrorBadRequest("Failed to parse JWT token."))?;

    let key_id = header.kid.ok_or_else(|| fail(&req))?;

    let redis_client = req.app_data::<Data<Client>>().unwrap();

    let mut redis = redis_client.get_connection().map_err(|e| {
        error!("Redis connection init failed: {:?}", e);
        ErrorInternalServerError("Internal server error.")
    })?;

    let pub_key = {
        let pem: String = redis
            .get(format!("baryonic:public_keys:{}", key_id))
            .map_err(|_| ErrorBadRequest("Invalid or expired key ID."))?;

        DecodingKey::from_ec_pem(pem.as_ref()).map_err(|e| {
            error!("{:?}", e);
            ErrorInternalServerError("Internal server error.")
        })?
    };

    let token_message =
        jsonwebtoken::decode::<Claims>(token, &pub_key, &Validation::new(Algorithm::ES384))
            .map_err(|_| ErrorBadRequest("Failed to parse or validate JWT token."))?;

    let key_ref_id: i64 = token_message
        .claims
        .sub
        .parse()
        .map_err(|_| ErrorBadRequest("Invalid JWT subject ID."))?;

    let id = redis
        .get::<_, Option<i64>>(format!("baryonic:jwt:{}", key_ref_id))
        .map_err(|e| {
            error!("{:?}", e);
            ErrorInternalServerError("Internal server error.")
        })?
        .ok_or_else(|| fail(&req))?
        .into();

    req.extensions_mut().insert(User { id, key_ref_id });

    Ok(req)
}

fn fail(req: &ServiceRequest) -> Error {
    let config = req
        .app_data::<Config>()
        .cloned()
        .unwrap_or_default()
        .scope("default");

    AuthenticationError::from(config).into()
}
