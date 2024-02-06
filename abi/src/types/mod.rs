mod reservation;
mod reservation_query;
mod reservation_status;

use std::ops::Bound;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;
pub use reservation::*;
pub use reservation_query::*;
use sqlx::postgres::types::PgRange;

use crate::{convert_to_utc_time, Error};

pub fn validate_range(start: Option<&Timestamp>, end: Option<&Timestamp>) -> Result<(), Error> {
    if start.is_none() || end.is_none() {
        return Err(Error::InvalidTime);
    }

    let start = start.unwrap().seconds;
    let end = end.unwrap().seconds;

    if start > end {
        return Err(Error::InvalidTime);
    }

    Ok(())
}

pub fn get_timespan(start: Option<&Timestamp>, end: Option<&Timestamp>) -> PgRange<DateTime<Utc>> {
    let start = convert_to_utc_time(start.unwrap());
    let end = convert_to_utc_time(end.unwrap());

    PgRange {
        start: Bound::Included(start),
        end: Bound::Excluded(end),
    }
}
