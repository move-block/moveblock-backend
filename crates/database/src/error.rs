use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("sqlx error: {:?}", &self)]
    SqlXError(sqlx::Error),

    #[error("var error")]
    VarError(std::env::VarError),
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Self::SqlXError(e)
    }
}

impl From<std::env::VarError> for Error {
    fn from(e: std::env::VarError) -> Self {
        Self::VarError(e)
    }
}
