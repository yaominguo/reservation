mod conflict;
use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

pub use conflict::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error")]
    DbError(sqlx::Error),

    #[error("Invalid start or end time for the reservation")]
    InvalidTime,

    #[error("Conflict reservation")]
    ConflictReservation(ReservationConflictInfo),

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),

    #[error("Unknown error")]
    Unknown,
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(db_err) => {
                let err: &PgDatabaseError = db_err.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        // convert err detail from str to ReservationConflictInfo
                        Error::ConflictReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Error::DbError(sqlx::Error::Database(db_err)),
                }
            }
            _ => Error::DbError(e),
        }
    }
}
