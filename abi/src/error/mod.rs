mod conflict;

use sqlx::postgres::PgDatabaseError;
use thiserror::Error;

pub use conflict::{ReservationConflictInfo, ReservationWindow};

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown error")]
    Unknown,

    #[error("Read config error")]
    ReadConfigError,

    #[error("Parse config error")]
    ParseConfigError,

    #[error("Database error")]
    DbError(sqlx::Error),

    #[error("Conflict reservation")]
    ConflictReservation(ReservationConflictInfo),

    #[error("No reservation found by the given condition")]
    NotFound,

    #[error("Invalid start time or end time for the reservation")]
    InvalidTime,

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid reservation id: {0}")]
    InvalidReservationId(i64),

    #[error("Invalid user id: {0}")]
    InvalidResourceId(String),

    #[error("Invalid status: {0}")]
    InvalidStatus(i32),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown, Self::Unknown) => true,
            (Self::ReadConfigError, Self::ReadConfigError) => true,
            (Self::ParseConfigError, Self::ParseConfigError) => true,
            (Self::DbError(_), Self::DbError(_)) => true,
            (Self::ConflictReservation(v1), Self::ConflictReservation(v2)) => v1 == v2,
            (Self::NotFound, Self::NotFound) => true,
            (Self::InvalidTime, Self::InvalidTime) => true,
            (Self::InvalidUserId(v1), Self::InvalidUserId(v2)) => v1 == v2,
            (Self::InvalidReservationId(v1), Self::InvalidReservationId(v2)) => v1 == v2,
            (Self::InvalidResourceId(v1), Self::InvalidResourceId(v2)) => v1 == v2,
            (Self::InvalidStatus(v1), Self::InvalidStatus(v2)) => v1 == v2,
            _ => false,
        }
    }
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

impl From<crate::Error> for tonic::Status {
    fn from(e: crate::Error) -> Self {
        match e {
            crate::Error::Unknown => tonic::Status::internal("Unknown error"),
            crate::Error::DbError(_) => tonic::Status::internal("Database error"),
            crate::Error::ReadConfigError => tonic::Status::internal("Read config error"),
            crate::Error::ParseConfigError => tonic::Status::internal("Parse config error"),
            crate::Error::ConflictReservation(info) => {
                let msg = format!("Conflict reservation: {:?}", info);
                tonic::Status::already_exists(msg)
            }
            crate::Error::NotFound => {
                tonic::Status::not_found("No reservation found by the given condition")
            }
            crate::Error::InvalidTime => tonic::Status::invalid_argument(
                "Invalid start time or end time for the reservation",
            ),
            crate::Error::InvalidUserId(_) => tonic::Status::invalid_argument("Invalid user id"),
            crate::Error::InvalidReservationId(_) => {
                tonic::Status::invalid_argument("Invalid reservation id")
            }
            crate::Error::InvalidResourceId(_) => {
                tonic::Status::invalid_argument("Invalid resource id")
            }
            crate::Error::InvalidStatus(_) => tonic::Status::invalid_argument("Invalid status"),
        }
    }
}
