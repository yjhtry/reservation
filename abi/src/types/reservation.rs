use chrono::{DateTime, FixedOffset, Utc};
use std::ops::Range;

use crate::{convert_to_timestamp, convert_to_utc_time};
use crate::{Error, Reservation, ReservationStatus};

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

    pub fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }

        if self.start.is_none() || self.end.is_none() {
            return Err(Error::InvalidTime);
        }

        let start = convert_to_utc_time(self.start.clone().unwrap()).ok_or(Error::InvalidTime)?;
        let end = convert_to_utc_time(self.end.clone().unwrap()).ok_or(Error::InvalidTime)?;

        if start > end {
            return Err(Error::InvalidTime);
        }

        Ok(())
    }
}
