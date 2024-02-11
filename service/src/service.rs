use reservation::Rsvp;

use abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, ConfirmRequest,
    ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse, ListenRequest,
    QueryRequest, ReserveRequest, ReserveResponse, UpdateRequest, UpdateResponse,
};
use tonic::{Request, Response, Status};

use crate::{ListenStream, ReservationStream, RsvpService, TonicReceiverStream};

#[tonic::async_trait]
impl ReservationService for RsvpService {
    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> std::result::Result<Response<ReserveResponse>, Status> {
        let request = request.into_inner();

        if request.reservation.is_none() {
            return Err(Status::invalid_argument("reservation is required"));
        }

        let reservation = self.manager.reserve(request.reservation.unwrap()).await?;

        Ok(Response::new(ReserveResponse {
            reservation: Some(reservation),
        }))
    }
    async fn confirm(
        &self,
        request: Request<ConfirmRequest>,
    ) -> std::result::Result<Response<ConfirmResponse>, Status> {
        let request = request.into_inner();
        if request.id == 0 {
            return Err(Status::invalid_argument("id is required"));
        }

        let reservation = self.manager.change_status(request.id).await?;

        Ok(Response::new(ConfirmResponse {
            reservation: Some(reservation),
        }))
    }
    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> std::result::Result<Response<UpdateResponse>, Status> {
        let request = request.into_inner();

        if request.id == 0 {
            return Err(Status::invalid_argument("id is required"));
        }

        let reservation = self.manager.update_note(request.id, request.note).await?;

        Ok(Response::new(UpdateResponse {
            reservation: Some(reservation),
        }))
    }
    async fn cancel(
        &self,
        request: Request<CancelRequest>,
    ) -> std::result::Result<Response<CancelResponse>, Status> {
        let request = request.into_inner();

        if request.id == 0 {
            return Err(Status::invalid_argument("id is required"));
        }

        let _ = self.manager.delete(request.id).await?;

        Ok(Response::new(CancelResponse { reservation: None }))
    }
    async fn get(
        &self,
        request: Request<GetRequest>,
    ) -> std::result::Result<Response<GetResponse>, Status> {
        let request = request.into_inner();

        if request.id == 0 {
            return Err(Status::invalid_argument("id is required"));
        }

        let reservation = self.manager.get(request.id).await?;

        Ok(Response::new(GetResponse {
            reservation: Some(reservation),
        }))
    }
    /// Server streaming response type for the query method.
    type queryStream = ReservationStream;
    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> std::result::Result<Response<Self::queryStream>, Status> {
        let request = request.into_inner();
        if request.query.is_none() {
            return Err(Status::invalid_argument("query is required"));
        }

        let rsvps = self.manager.query(request.query.unwrap()).await;

        let stream = TonicReceiverStream::new(rsvps);
        Ok(Response::new(Box::pin(stream)))
    }
    async fn filter(
        &self,
        request: Request<FilterRequest>,
    ) -> std::result::Result<Response<FilterResponse>, Status> {
        let request = request.into_inner();

        if request.filter.is_none() {
            return Err(Status::invalid_argument("filter is required"));
        }

        let (pager, reservations) = self.manager.filter(request.filter.unwrap()).await?;

        Ok(Response::new(FilterResponse {
            pager: Some(pager),
            reservations,
        }))
    }
    /// Server streaming response type for the listen method.
    type listenStream = ListenStream;
    /// another system can monitor the reservations and newly reserved/confirmed/canceled reservations
    async fn listen(
        &self,
        _request: Request<ListenRequest>,
    ) -> std::result::Result<Response<Self::listenStream>, Status> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi::{Config, Reservation};
    use lazy_static::lazy_static;
    use sqlx::{types::Uuid, Connection, Executor, PgConnection};
    use std::{ops::Deref, sync::Arc, thread};
    use tokio::runtime::Runtime;

    lazy_static! {
        static ref TEST_RT: Runtime = Runtime::new().unwrap();
    }

    struct TestConfig {
        config: Arc<Config>,
    }

    impl Deref for TestConfig {
        type Target = Config;

        fn deref(&self) -> &Self::Target {
            &self.config
        }
    }

    impl TestConfig {
        pub fn new() -> Self {
            let mut config = Config::load("../service/fixtures/config.yaml").unwrap();
            let uuid = Uuid::new_v4();
            let dbname = format!("test_{}", uuid);
            config.db.dbname = dbname.clone();

            let server_url = config.db.server_url();
            let url = config.db.url();

            // create database dbname
            thread::spawn(move || {
                TEST_RT.block_on(async move {
                    // use server url to create database
                    let mut conn = PgConnection::connect(&server_url).await.unwrap();
                    conn.execute(format!(r#"CREATE DATABASE "{}""#, dbname).as_str())
                        .await
                        .unwrap();

                    // now connect to test database for migration
                    let mut conn = PgConnection::connect(&url).await.unwrap();
                    sqlx::migrate!("../migrations")
                        .run(&mut conn)
                        .await
                        .unwrap();
                });
            })
            .join()
            .expect("failed to create database");

            Self {
                config: Arc::new(config),
            }
        }
    }

    impl Drop for TestConfig {
        fn drop(&mut self) {
            let server_url = self.db.server_url();
            let dbname = self.config.db.dbname.clone();

            thread::spawn(move || {
                TEST_RT.block_on(async move {
                    let mut conn = sqlx::PgConnection::connect(&server_url).await.unwrap();
                    // terminate existing connections
                    sqlx::query(&format!(r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND datname = '{}'"#, dbname))
                    .execute(&mut conn)
                    .await
                    .expect("Terminate all other connections");

                    conn.execute(format!(r#"DROP DATABASE "{}""#, dbname).as_str())
                        .await
                        .expect("Error while querying the drop database");
                });
            })
            .join()
            .expect("failed to drop database");
        }
    }

    #[tokio::test]
    async fn rpc_reserve_should_work() {
        let config = TestConfig::new();

        let service = RsvpService::from_config(&config).await.unwrap();
        let reservation = Reservation::new_pending(
            "john",
            "room_01",
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "I need this room for a meeting",
        );
        let request = tonic::Request::new(ReserveRequest {
            reservation: Some(reservation.clone()),
        });
        let response = service.reserve(request).await.unwrap();
        let reservation1 = response.into_inner().reservation;
        assert!(reservation1.is_some());
        let reservation1 = reservation1.unwrap();
        assert_eq!(reservation1.user_id, reservation.user_id);
        assert_eq!(reservation1.resource_id, reservation.resource_id);
        assert_eq!(reservation1.start, reservation.start);
        assert_eq!(reservation1.end, reservation.end);
        assert_eq!(reservation1.note, reservation.note);
        assert_eq!(reservation1.status, reservation.status);
    }
}
