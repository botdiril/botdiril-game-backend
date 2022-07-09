use std::fmt::Debug;

use actix_web::error::ErrorInternalServerError;
use actix_web::http::StatusCode;
use actix_web::{error, HttpResponse};
use log::error;
use serde_json::json;

use crate::game_api::game_struct::GameError;

#[derive(Debug)]
pub struct CommandError(actix_web::Error);

impl From<actix_web::Error> for CommandError {
    fn from(err: actix_web::Error) -> CommandError {
        CommandError(err)
    }
}

impl From<GameError> for CommandError {
    fn from(err: GameError) -> CommandError {
        CommandError(err.into())
    }
}

impl From<mongodb::error::Error> for CommandError {
    fn from(err: mongodb::error::Error) -> CommandError {
        error!("MongoDB error: {:?}", err);
        CommandError(ErrorInternalServerError("Internal server error."))
    }
}

impl From<CommandError> for actix_web::Error {
    fn from(err: CommandError) -> Self {
        err.0
    }
}

pub type CommandResult<T> = actix_web::Result<T, CommandError>;

impl error::ResponseError for GameError {
    fn status_code(&self) -> StatusCode {
        match *self {
            GameError::NotEnough{..} => StatusCode::UNPROCESSABLE_ENTITY,
            GameError::IllegalAction => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "errors": [self]
        }))
    }
}
