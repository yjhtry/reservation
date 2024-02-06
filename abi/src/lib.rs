mod error;
mod pb;
mod types;
mod utils;

pub use error::*;
pub use pb::*;
pub use types::*;
pub use utils::*;

pub trait Validator {
    fn validate(&self) -> Result<(), Error>;
}

impl From<RsvpStatus> for ReservationStatus {
    fn from(status: RsvpStatus) -> Self {
        match status {
            RsvpStatus::Unknown => Self::Unknown,
            RsvpStatus::Pending => Self::Pending,
            RsvpStatus::Confirmed => Self::Confirmed,
            RsvpStatus::Blocked => Self::Blocked,
        }
    }
}
