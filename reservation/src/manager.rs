use futures::StreamExt;
use tokio::sync::mpsc;

use abi::{convert_to_utc_time, ReservationStatus, Validator};
use async_trait::async_trait;
use sqlx::{Either, Row};

use crate::{Error, ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, Error> {
        rsvp.validate()?;

        let mut return_rsvp = rsvp.clone();

        let timespan = rsvp.get_timespan();

        let status = ReservationStatus::try_from(rsvp.status).unwrap_or(ReservationStatus::Pending);

        let id = sqlx::query(
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

        return_rsvp.id = id;

        Ok(return_rsvp)
    }

    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        id.validate()?;

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
        id.validate()?;

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

        id.validate()?;

        let _ = sqlx::query("DELETE FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, Error> {
        // get reservation by id

        id.validate()?;

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

    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> mpsc::Receiver<Result<abi::Reservation, abi::Error>> {
        let start = query.start.map(|v| convert_to_utc_time(&v));
        let end = query.end.map(|v| convert_to_utc_time(&v));
        let status =
            ReservationStatus::try_from(query.status).unwrap_or(ReservationStatus::Pending);

        let (tx, rx) = mpsc::channel(128);

        let mut rsvps = sqlx::query_as(
            "SELECT * FROM rsvp.query($1, $2, $3, $4, $5::rsvp.reservation_status, $6, $7, $8)",
        )
        .bind(str_to_option(&query.user_id))
        .bind(str_to_option(&query.resource_id))
        .bind(start)
        .bind(end)
        .bind(status.to_string())
        .bind(query.is_desc)
        .bind(query.page)
        .bind(query.page_size)
        .fetch_many(&self.pool);

        while let Some(ret) = rsvps.next().await {
            match ret {
                Ok(Either::Left(r)) => {
                    println!("Query result: {:?}", r);
                }
                Ok(Either::Right(r)) => {
                    if tx.send(Ok(r)).await.is_err() {
                        // rx is dropped, so client disconnected
                        break;
                    }
                }
                Err(e) => {
                    println!("Query error: {:?}", e);
                    if tx.send(Err(e.into())).await.is_err() {
                        // rx is dropped, so client disconnected
                        break;
                    }
                }
            }
        }

        rx
    }

    async fn filter(
        &self,
        query: abi::ReservationFilter,
    ) -> Result<(abi::FilterPager, Vec<abi::Reservation>), Error> {
        let status =
            ReservationStatus::try_from(query.status).unwrap_or(ReservationStatus::Pending);

        let mut rsvps: Vec<abi::Reservation> = sqlx::query_as(
            "SELECT * FROM rsvp.filter($1, $2, $3::rsvp.reservation_status, $4, $5, $6)",
        )
        .bind(str_to_option(&query.user_id))
        .bind(str_to_option(&query.resource_id))
        .bind(status.to_string())
        .bind(query.cursor)
        .bind(query.is_desc)
        .bind(query.page_size)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query("SELECT COUNT(*) FROM rsvp.reservations")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>(0);

        if query.is_desc {
            rsvps.reverse();
        }

        let mut prev = -1;
        let mut next = -1;

        if let Some(start) = rsvps.first() {
            prev = start.id;
        }

        if rsvps.len() == (query.page_size as usize) {
            if let Some(end) = rsvps.last() {
                next = end.id;
            }
        }

        let pager = abi::FilterPager {
            prev,
            next,
            // TODO optimize total sum
            total,
        };

        Ok((pager, rsvps))
    }
}

fn str_to_option(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use abi::Reservation;
    use abi::ReservationConflictInfo;
    use abi::ReservationWindow;
    use prost_types::Timestamp;

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

        assert!(rsvp.id != 0);
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
        let rsvp = manager.change_status(rsvp.id).await.unwrap();

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
        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        let rsvp = manager.change_status(rsvp.id).await.unwrap_err();

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
        let rsvp = manager.update_note(rsvp.id, note.clone()).await.unwrap();

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
        manager.delete(rsvp.id).await.unwrap();

        let rsvp = manager.get(rsvp.id).await.unwrap_err();

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
        let insert = manager.get(rsvp.id).await.unwrap();

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

        let rsvp = manager.reserve(insert).await.unwrap();

        let query = abi::ReservationQueryBuilder::default()
            .user_id("john")
            // .resource_id("ocean_view_room_3")
            .start("2024-01-01T00:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2024-01-09T00:00:00-0700".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();

        let mut rx = manager.query(query).await;
        assert_eq!(rx.recv().await, Some(Ok(rsvp.clone())));
        assert_eq!(rx.recv().await, None);

        // if the window is not match, should return empty
        let query = abi::ReservationQueryBuilder::default()
            .user_id("john")
            .resource_id("ocean_view_room_3")
            .start("2024-01-01T00:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2024-01-02T00:00:00-0700".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();

        let mut rx = manager.query(query).await;
        assert_eq!(rx.recv().await, None);

        // if the status is not match, should return empty
        let query = abi::ReservationQueryBuilder::default()
            .user_id("john")
            .resource_id("ocean_view_room_3")
            .start("2024-01-01T00:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2024-01-09T00:00:00-0700".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Confirmed as i32)
            .build()
            .unwrap();

        let mut rx = manager.query(query.clone()).await;
        assert_eq!(rx.recv().await, None);

        // change status to confirmed, should return the reservation
        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        let mut rx = manager.query(query).await;

        assert_eq!(rx.recv().await, Some(Ok(rsvp)));
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn filter_reservation_should_work() {
        let manager = ReservationManager::new(migrated_pool);
        let insert = Reservation::new_pending(
            "john",
            "ocean_view_room_3",
            "2024-01-01T00:00:00-0700".parse().unwrap(),
            "2024-01-03T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp1 = manager.reserve(insert).await.unwrap();

        let filter = abi::ReservationFilterBuilder::default()
            .user_id("john")
            .cursor(0)
            .build()
            .unwrap();

        let (_, rsvps) = manager.filter(filter).await.unwrap();

        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvp1, rsvps[0]);

        // if the cursor is not match, should return empty
        let filter = abi::ReservationFilterBuilder::default()
            .user_id("john")
            .cursor(10)
            .resource_id("ocean_view_room_3")
            .build()
            .unwrap();

        let (_, rsvps) = manager.filter(filter).await.unwrap();

        assert_eq!(rsvps.len(), 0);

        // if the cursor is match, but the desc is not match, should return empty
        let filter = abi::ReservationFilterBuilder::default()
            .user_id("john")
            .cursor(0)
            .is_desc(true)
            .status(ReservationStatus::Confirmed as i32)
            .build()
            .unwrap();

        let (_, rsvps) = manager.filter(filter).await.unwrap();

        assert_eq!(rsvps.len(), 0);

        // skip the first reservation, should return second reservation
        let insert = Reservation::new_pending(
            "john",
            "ocean_view_room_3",
            "2024-01-05T00:00:00-0700".parse().unwrap(),
            "2024-01-06T00:00:00-0700".parse().unwrap(),
            "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
        );

        let rsvp2 = manager.reserve(insert).await.unwrap();

        let filter = abi::ReservationFilterBuilder::default()
            .user_id("john")
            .cursor(1)
            .is_desc(false)
            .build()
            .unwrap();

        let (_, rsvps) = manager.filter(filter).await.unwrap();

        assert_eq!(rsvp2, rsvps[0]);

        // if the is_desc is true and the cursor is 2, should return the first reservation
        let filter = abi::ReservationFilterBuilder::default()
            .user_id("john")
            .cursor(2)
            .is_desc(true)
            .build()
            .unwrap();

        let (_, rsvps) = manager.filter(filter).await.unwrap();

        assert_eq!(rsvp1, rsvps[0]);
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn filter_pager_should_work() {
        let manager = ReservationManager::new(migrated_pool);
        let mut reservation_list = vec![];

        for i in 1..20 {
            let insert = Reservation::new_pending(
                "john",
                "ocean_view_room_3",
                format!("2024-01-{:02}T00:00:00-0700", i).parse().unwrap(),
                format!("2024-01-{:02}T00:00:00-0700", i + 1)
                    .parse()
                    .unwrap(),
                "I'll arrive at 3pm, Please help to upgrade to executive room if possible.",
            );

            reservation_list.push(manager.reserve(insert).await.unwrap());
        }

        let filter = abi::ReservationFilterBuilder::default()
            .user_id("john")
            .cursor(2)
            .build()
            .unwrap();

        let (pager, _) = manager.filter(filter).await.unwrap();

        assert_eq!(pager.prev, 3);
        assert_eq!(pager.next, 12);

        let filter = abi::ReservationFilterBuilder::default()
            .user_id("john")
            .cursor(pager.next)
            .build()
            .unwrap();

        let (pager, _) = manager.filter(filter).await.unwrap();

        assert_eq!(pager.prev, 13);
        assert_eq!(pager.next, -1);

        let filter = abi::ReservationFilterBuilder::default()
            .user_id("john")
            .cursor(pager.prev)
            .is_desc(true)
            .build()
            .unwrap();

        let (pager, _) = manager.filter(filter).await.unwrap();

        assert_eq!(pager.prev, 3);
        assert_eq!(pager.next, 12);
    }
}
