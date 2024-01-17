mod error;
mod pb;

use chrono::{DateTime, FixedOffset, Utc};
use chrono::{NaiveDateTime, TimeZone};
pub use error::*;
pub use pb::*;
use prost_types::Timestamp;
use std::fmt;
use std::ops::Range;

pub fn convert_to_utc_time(ts: Timestamp) -> Option<DateTime<Utc>> {
    let naive_datetime = NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32);
    naive_datetime.map(|ndt| Utc.from_utc_datetime(&ndt))
}

pub fn convert_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as _,
    }
}

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::Unknown => write!(f, "unknown"),
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Blocked => write!(f, "blocked"),
        }
    }
}

impl Reservation {
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id: uid.into(),
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
            status: ReservationStatus::Pending as i32,
        }
    }

    pub fn get_timespan(&self) -> Range<DateTime<Utc>> {
        let start = convert_to_utc_time(self.start.clone().unwrap()).unwrap();
        let end = convert_to_utc_time(self.end.clone().unwrap()).unwrap();

        start..end
    }

    pub fn validate(&self) -> Result<(), ReservationError> {
        if self.user_id.is_empty() {
            return Err(ReservationError::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(ReservationError::InvalidResourceId(
                self.resource_id.clone(),
            ));
        }

        if self.start.is_none() || self.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let start = convert_to_utc_time(self.start.clone().unwrap())
            .ok_or(ReservationError::InvalidTime)?;
        let end =
            convert_to_utc_time(self.end.clone().unwrap()).ok_or(ReservationError::InvalidTime)?;

        if start > end {
            return Err(ReservationError::InvalidTime);
        }

        Ok(())
    }
}
