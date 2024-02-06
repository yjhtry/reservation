use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{get_timespan, validate_range, Error, ReservationQuery, ReservationStatus, Validator};

impl Validator for ReservationQuery {
    fn validate(&self) -> Result<(), Error> {
        ReservationStatus::try_from(self.status).map_err(|_| Error::InvalidStatus(self.status))?;

        validate_range(self.start.as_ref(), self.end.as_ref())?;

        Ok(())
    }
}

impl ReservationQuery {
    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        get_timespan(self.start.as_ref(), self.end.as_ref())
    }
}
