use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("unknown error")]
    Unknown,

    #[error("Database error")]
    DbError(#[from] sqlx::Error),

    #[error("Invalid start time or end time for the reservation")]
    InvalidTime,

    #[error("Invalid user id: {0}")]
    InvalidUserId(String),

    #[error("Invalid user id: {0}")]
    InvalidResourceId(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

// impl From<sqlx::Error> for ReservationError {
//     fn from(err: sqlx::Error) -> Self {
//         match err {
//             sqlx::Error::Database(e) => match e.code() {
//                 Some(code) => match code {
//                     "23503" => Self::InvalidUserId("".to_string()),
//                     _ => Self::DbError(err),
//                 },
//                 None => Self::DbError(err),
//             },
//             _ => Self::DbError(err),
//         }
//     }
// }
