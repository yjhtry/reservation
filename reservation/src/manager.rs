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

use crate::{Error, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, Error> {
        rsvp.validate()?;

        let mut return_rsvp = rsvp.clone();

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();

        let status = ReservationStatus::try_from(rsvp.status).unwrap_or(ReservationStatus::Pending);

        println!("status: {:?}", rsvp.status.to_string());

        let id: Uuid = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status)
            VALUES ($1, $2, $3, $4, $5::rsvp.reservations_status)
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

    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        let id = Uuid::parse_str(&id).map_err(|e| Error::InvalidReservationId(e.to_string()))?;

        // if current status is pending, change is to confirmed, otherwise do nothing
        let rsvp: abi::Reservation = sqlx::query_as(
            "UPDATE rsvp.reservations
            SET status = 'confirmed'::rsvp.reservations_status
            WHERE id = $1 AND status = 'pending'::rsvp.reservations_status
            RETURNING *",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
    }

    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, Error> {
        let id = Uuid::parse_str(&id).map_err(|e| Error::InvalidReservationId(e.to_string()))?;

        // change note for the reservation
        let rsvp: abi::Reservation = sqlx::query_as(
            "UPDATE rsvp.reservations
            SET note = $2
            WHERE id = $1
            RETURNING *",
        )
        .bind(id)
        .bind(note)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
    }

    async fn delete(&self, id: ReservationId) -> Result<(), Error> {
        // delete reservation by id

        let id = Uuid::parse_str(&id).map_err(|e| Error::InvalidReservationId(e.to_string()))?;

        let _ = sqlx::query("DELETE FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        // get reservation by id

        let id = Uuid::parse_str(&id).map_err(|e| Error::InvalidReservationId(e.to_string()))?;

        let rsvp: abi::Reservation = sqlx::query_as(
            "SELECT id, user_id, resource_id, timespan, note, status
            FROM rsvp.reservations
            WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
    }

    async fn query(&self, _query: abi::ReservationQuery) -> Result<Vec<abi::Reservation>, Error> {
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

    use super::*;
    use abi::Reservation;
    use abi::ReservationConflictInfo;
    use abi::ReservationWindow;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool);

        let rsvp = Reservation::new_pending(
            "john",
            "ocean_view_room_1",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();

        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_conflict_reservation_should_reject() {
        let manager = ReservationManager::new(migrated_pool);

        let rsvp1 = Reservation::new_pending(
            "john",
            "ocean_view_room_2",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp2 = Reservation::new_pending(
            "lei",
            "ocean_view_room_2",
            "2024-01-02T00:00:00-0700".parse().unwrap(),
            "2024-01-04T00:00:00-0700".parse().unwrap(),
            "Hello, I'm Lei, Please help to upgrade to executive room if possible.",
        );

        let _ = manager.reserve(rsvp1).await.unwrap();
        let err = manager.reserve(rsvp2).await.unwrap_err();

        let info =
            Error::ConflictReservation(ReservationConflictInfo::Parsed(ReservationWindow::new(
                "ocean_view_room_2".to_string(),
                "2024-01-01T07:00:00+00:00".parse().unwrap(),
                "2024-01-03T07:00:00+00:00".parse().unwrap(),
            )));

        assert_eq!(err, info);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn change_reservation_status_should_work() {
        let manager = ReservationManager::new(migrated_pool);
        let rsvp = Reservation::new_pending(
            "john",
            "ocean_view_room_3",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        let rsvp = manager.change_status(rsvp.id.clone()).await.unwrap();

        assert_eq!(rsvp.status, ReservationStatus::Confirmed as i32);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn change_reservation_not_pending_should_nothing() {
        let manager = ReservationManager::new(migrated_pool);
        let rsvp = Reservation::new_pending(
            "john",
            "ocean_view_room_3",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        let rsvp = manager.change_status(rsvp.id.clone()).await.unwrap();
        let rsvp = manager.change_status(rsvp.id.clone()).await.unwrap_err();

        assert_eq!(rsvp, Error::NotFound);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let manager = ReservationManager::new(migrated_pool);
        let rsvp = Reservation::new_pending(
            "john",
            "ocean_view_room_3",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let note = "new note".to_string();

        let rsvp = manager.reserve(rsvp).await.unwrap();
        let rsvp = manager
            .update_note(rsvp.id.clone(), note.clone())
            .await
            .unwrap();

        assert_eq!(rsvp.note, note);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn delete_reservation_by_id_should_work() {
        let manager = ReservationManager::new(migrated_pool);
        let rsvp = Reservation::new_pending(
            "john",
            "ocean_view_room_3",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        manager.delete(rsvp.id.clone()).await.unwrap();
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn get_reservation_by_id_should_work() {
        let manager = ReservationManager::new(migrated_pool);
        let insert = Reservation::new_pending(
            "john",
            "ocean_view_room_3",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp = manager.reserve(insert).await.unwrap();
        let insert = manager.get(rsvp.id.clone()).await.unwrap();

        assert_eq!(insert, rsvp);
    }
}
