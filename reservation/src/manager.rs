use abi::ReservationStatus;
use async_trait::async_trait;
use sqlx::{
    postgres::types::PgRange,
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
    PgPool, Row,
};

use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, ReservationError> {
        rsvp.validate()?;

        let mut return_rsvp = rsvp.clone();

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();

        let status = ReservationStatus::try_from(rsvp.status).unwrap_or(ReservationStatus::Pending);

        println!("status: {:?}", rsvp.status.to_string());

        let id: Uuid = sqlx::query(
            "INSERT INTO rsvp.reservation (user_id, resource_id, timespan, note, status)
            VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status)
            RETURNING id",
        )
        .bind(rsvp.user_id)
        .bind(rsvp.resource_id)
        .bind(timespan)
        .bind(rsvp.note)
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?
        .get(0);

        return_rsvp.id = id.to_string();

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

    use abi::Reservation;

    use super::*;
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool);

        let rsvp = Reservation::new_pending(
            "user",
            "resource",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        assert!(!rsvp.id.is_empty());
    }
}
