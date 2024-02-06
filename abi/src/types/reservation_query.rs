use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{
    convert_to_timestamp, get_timespan, validate_range, Error, ReservationQuery, ReservationStatus,
    Validator,
};

impl Validator for ReservationQuery {
    fn validate(&self) -> Result<(), Error> {
        ReservationStatus::try_from(self.status).map_err(|_| Error::InvalidStatus(self.status))?;

        validate_range(self.start.as_ref(), self.end.as_ref())?;

        Ok(())
    }
}

pub struct ReservationQueryParams {
    pub uid: String,
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub status: ReservationStatus,
    pub page: i32,
    pub page_size: i32,
    pub is_desc: bool,
}

impl ReservationQuery {
    pub fn new(params: ReservationQueryParams) -> Self {
        let ReservationQueryParams {
            uid,
            rid,
            start,
            end,
            status,
            page,
            page_size,
            is_desc,
        } = params;
        Self {
            user_id: uid,
            resource_id: rid,
            start: Some(convert_to_timestamp(start)),
            end: Some(convert_to_timestamp(end)),
            status: status as i32,
            page,
            page_size,
            is_desc,
        }
    }

    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        get_timespan(self.start.as_ref(), self.end.as_ref())
    }
}
