use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("unknown error")]
    Unknown,
}
