use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to setup default files")]
    Setup {},

    #[error("Not found: {msg}")]
    NotFound { msg: String },

    #[error("IO Error: {msg}")]
    IoError { msg: String },

    #[error("Failed to generate {msg}")]
    Generate { msg: String },

    #[error("{0:?}")]
    AnyError(anyhow::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError { msg: e.to_string() }
    }
}
