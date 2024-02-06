use chrono::{DateTime, FixedOffset, Utc};
use sqlx::postgres::types::PgRange;
use sqlx::types::Uuid;
use sqlx::Row;
use sqlx::{postgres::PgRow, FromRow};
use std::ops::{Bound, Range};

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

// impl sqxl::FromRow for Reservation

struct NativeRange<T> {
    start: Option<T>,
    end: Option<T>,
}

impl<T> From<PgRange<T>> for NativeRange<T> {
    fn from(range: PgRange<T>) -> Self {
        let f = |t: Bound<T>| match t {
            Bound::Included(t) => Some(t),
            Bound::Excluded(t) => Some(t),
            Bound::Unbounded => None,
        };

        Self {
            start: f(range.start),
            end: f(range.end),
        }
    }
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}

impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let timespan: PgRange<DateTime<Utc>> = row.get("timespan");
        let timespan: NativeRange<DateTime<Utc>> = timespan.into();

        // in real word, reservation must have a start time
        assert!(timespan.start.is_some());
        assert!(timespan.end.is_some());

        let start = timespan.start.unwrap();
        let end = timespan.end.unwrap();

        let id: Uuid = row.get("id");
        let status: RsvpStatus = row.get("status");

        Ok(Self {
            id: Uuid::to_string(&id),
            user_id: row.get("user_id"),
            resource_id: row.get("resource_id"),
            start: Some(convert_to_timestamp(start)),
            end: Some(convert_to_timestamp(end)),
            note: row.get("note"),
            status: ReservationStatus::from(status) as i32,
        })
    }
}
