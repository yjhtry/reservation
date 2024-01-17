mod pb;

use chrono::{DateTime, Utc};
use chrono::{NaiveDateTime, TimeZone};
pub use pb::*;
use prost_types::Timestamp;
use std::fmt;

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
