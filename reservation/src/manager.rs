use abi::convert_to_utc_time;
use async_trait::async_trait;
use sqlx::{
    postgres::types::PgRange,
    types::chrono::{DateTime, Utc},
    PgPool, Row,
};

use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let mut return_rsvp = rsvp.clone();

        let start =
            convert_to_utc_time(rsvp.start.unwrap()).ok_or(ReservationError::InvalidTime)?;
        let end = convert_to_utc_time(rsvp.end.unwrap()).ok_or(ReservationError::InvalidTime)?;

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();

        let id = sqlx::query(
            "INSERT INTO reservation (user_id, resource_id, timespan, note, status)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id",
        )
        .bind(rsvp.user_id)
        .bind(rsvp.resource_id)
        .bind(timespan)
        .bind(rsvp.note)
        .bind(rsvp.status)
        .fetch_one(&self.pool)
        .await?
        .get(0);

        return_rsvp.id = id;

        Ok(return_rsvp)
    }

    async fn change_status(
        &self,
        _reservation_id: ReservationId,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn update_note(
        &self,
        _reservation_id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn delete(&self, _reservation_id: ReservationId) -> Result<(), ReservationError> {
        todo!()
    }

    async fn get(
        &self,
        _reservation_id: ReservationId,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        todo!()
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {

    use abi::convert_to_timestamp;
    use sqlx::types::chrono::FixedOffset;

    use super::*;
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let _manager = ReservationManager::new(migrated_pool);

        let start: DateTime<FixedOffset> = "2024-01-01T00:00:00-0700".parse().unwrap();
        let end: DateTime<FixedOffset> = "2024-01-03T00:00:00-0700".parse().unwrap();

        let _rsvp = abi::Reservation {
            id: "".to_string(),
            user_id: "user".to_string(),
            resource_id: "resource".to_string(),
            start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: "I'll arrive at 3pm, Please help to upgrade to executive room if possible."
                .to_string(),
            status: abi::ReservationStatus::Pending as i32,
        };
    }
}
