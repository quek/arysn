use thiserror::Error;

pub type Result<T> = std::result::Result<T, ArysnError>;

#[derive(Error, Debug)]
pub enum ArysnError {
    #[error("not found!")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<tokio_postgres::error::Error> for ArysnError {
    fn from(error: tokio_postgres::error::Error) -> Self {
        ArysnError::Other(error.into())
    }
}

impl From<std::io::Error> for ArysnError {
    fn from(error: std::io::Error) -> Self {
        ArysnError::Other(error.into())
    }
}
