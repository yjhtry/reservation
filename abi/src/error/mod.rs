use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown error")]
    Unknown,

    #[error("Database error")]
    DbError(sqlx::Error),

    #[error("ConflictError: {0}")]
    ConflictReservation(String),

    #[error("No reservation found by the given condition")]
    NotFound,

    #[error("Invalid start time or end time for the reservation")]
    InvalidTime,

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid user id: {0}")]
    InvalidResourceId(String),
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            sqlx::Error::RowNotFound => Error::NotFound,
            _ => Error::DbError(e),
        }
    }
}
