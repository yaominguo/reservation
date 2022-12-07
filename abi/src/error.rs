use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error")]
    DbError(#[from] sqlx::Error),

    #[error("Invalid start or end time for the reservation")]
    InvalidTime,

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("Unknown error")]
    Unknown,
}
