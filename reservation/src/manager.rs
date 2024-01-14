use abi::convert_to_utc_time;
use async_trait::async_trait;
use sqlx::{
    postgres::types::PgRange,
    types::chrono::{DateTime, Utc},
    Row,
};

use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let mut new_rsvp = rsvp.clone();

        let start = convert_to_utc_time(rsvp.start.unwrap());
        let end = convert_to_utc_time(rsvp.end.unwrap());

        if start.is_none() || end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let start = start.unwrap();
        let end = end.unwrap();

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

        new_rsvp.id = id;

        Ok(new_rsvp)
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
