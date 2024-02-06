use abi::{ReservationStatus, Validator};
use async_trait::async_trait;
use sqlx::{types::Uuid, PgPool, Row};

use crate::{Error, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, Error> {
        rsvp.validate()?;

        let mut return_rsvp = rsvp.clone();

        let timespan = rsvp.get_timespan();

        let status = ReservationStatus::try_from(rsvp.status).unwrap_or(ReservationStatus::Pending);

        let id: Uuid = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status)
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

    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        let id = Uuid::parse_str(&id).map_err(|e| Error::InvalidReservationId(e.to_string()))?;

        // if current status is pending, change is to confirmed, otherwise do nothing
        let rsvp: abi::Reservation = sqlx::query_as(
            "UPDATE rsvp.reservations
            SET status = 'confirmed'::rsvp.reservation_status
            WHERE id = $1 AND status = 'pending'::rsvp.reservation_status
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

    async fn query(&self, query: abi::ReservationQuery) -> Result<Vec<abi::Reservation>, Error> {
        let timespan = query.get_timespan();
        let status =
            ReservationStatus::try_from(query.status).unwrap_or(ReservationStatus::Pending);

        let rsvps: Vec<abi::Reservation> = sqlx::query_as(
            "SELECT * FROM rsvp.query($1, $2, $3, $4::rsvp.reservation_status, $5, $6, $7)",
        )
        .bind(str_to_option(&query.user_id))
        .bind(str_to_option(&query.resource_id))
        .bind(timespan)
        .bind(status.to_string())
        .bind(query.is_desc)
        .bind(query.page)
        .bind(query.page_size)
        .fetch_all(&self.pool)
        .await?;

        Ok(rsvps)
    }
}

fn str_to_option(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
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
    use abi::ReservationQueryParams;
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

        let rsvp = manager.get(rsvp.id.clone()).await.unwrap_err();

        assert_eq!(rsvp, Error::NotFound);
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

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn query_reservation_should_work() {
        let manager = ReservationManager::new(migrated_pool);
        let insert = Reservation::new_pending(
            "john",
            "ocean_view_room_3",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp1 = manager.reserve(insert).await.unwrap();

        let insert = Reservation::new_pending(
            "john",
            "ocean_view_room_3",
            "2024-01-04T00:00:00-0700".parse().unwrap(),
            "2024-01-06T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp2 = manager.reserve(insert).await.unwrap();

        let query = abi::ReservationQuery::new(ReservationQueryParams {
            uid: "john".to_string(),
            rid: "ocean_view_room_3".to_string(),
            start: "2024-01-01T00:00:00-0700".parse().unwrap(),
            end: "2024-01-09T00:00:00-0700".parse().unwrap(),
            status: ReservationStatus::Pending,
            page: 1,
            page_size: 10,
            is_desc: false,
        });

        let rsvps = manager.query(query).await.unwrap();

        assert_eq!(rsvps.len(), 2);
        assert_eq!(rsvp1, rsvps[0]);
        assert_eq!(rsvp2, rsvps[1]);
    }
}
