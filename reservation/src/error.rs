use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("unknown error")]
    Unknown,
    #[error("Invalid start time or end time for the reservation")]
    InvalidTime,
    #[error("Database error")]
    DbError(#[from] sqlx::Error),
}
