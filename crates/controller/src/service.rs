pub mod account;
pub mod block_stack;
pub mod domain;
pub mod function;

use actix_http::StatusCode;
use actix_web::ResponseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0:?}")]
    ActixWeb(actix_web::Error),

    #[error("{0:?}")]
    DbError(database::error::Error),

    #[error("{0:?}")]
    AnyError(anyhow::Error),

    #[error("UnAuthorized")]
    UnAuthorized {},

    #[error("NotFound: {msg}")]
    NotFound { msg: String },

    #[error("InvalidParams: {msg}")]
    InvalidParams { msg: String },

    #[error("Compile failed: {msg}")]
    CompileError { msg: String },
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ActixWeb(e) => e.error_response().status(),
            Error::AnyError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::UnAuthorized {} => StatusCode::UNAUTHORIZED,
            Error::NotFound { .. } => StatusCode::NOT_FOUND,
            Error::InvalidParams { .. } => StatusCode::BAD_REQUEST,
            Error::CompileError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<actix_web::Error> for Error {
    fn from(error: actix_web::Error) -> Self {
        Self::ActixWeb(error)
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Self::AnyError(e)
    }
}
