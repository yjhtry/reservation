mod error;

use async_trait::async_trait;
pub use error::ReservationError;

type ReservationId = String;

#[async_trait]
pub trait Rsvp {
    /// create a reservation
    async fn reservation(
        &self,
        reserve: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError>;

    /// change reservation status (if current status is pending, change it to confirmed)
    async fn change_status(
        &self,
        reservation_id: ReservationId,
    ) -> Result<abi::Reservation, ReservationError>;

    /// update reservation note
    async fn update_note(
        &self,
        reservation_id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, ReservationError>;

    /// delete reservation
    async fn delete(&self, reservation_id: ReservationId) -> Result<(), ReservationError>;

    /// get reservation by id
    async fn get(
        &self,
        reservation_id: ReservationId,
    ) -> Result<abi::Reservation, ReservationError>;

    /// get all reservations
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError>;
}
