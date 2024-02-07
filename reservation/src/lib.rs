mod manager;

use abi::{Error, ReservationId};
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Debug)]
pub struct ReservationManager {
    pub pool: PgPool,
}

#[async_trait]
pub trait Rsvp {
    /// create a reservation
    async fn reserve(&self, reserve: abi::Reservation) -> Result<abi::Reservation, Error>;

    /// change reservation status (if current status is pending, change it to confirmed)
    async fn change_status(&self, reservation_id: ReservationId)
        -> Result<abi::Reservation, Error>;

    /// update reservation note
    async fn update_note(
        &self,
        reservation_id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, Error>;

    /// delete reservation
    async fn delete(&self, reservation_id: ReservationId) -> Result<(), Error>;

    /// get reservation by id
    async fn get(&self, reservation_id: ReservationId) -> Result<abi::Reservation, Error>;

    /// get all reservations
    async fn query(&self, query: abi::ReservationQuery) -> Result<Vec<abi::Reservation>, Error>;

    /// get reservations order by id
    async fn filter(&self, query: abi::ReservationFilter) -> Result<Vec<abi::Reservation>, Error>;
}
