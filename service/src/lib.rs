use anyhow::Error;
use futures::stream::Stream;
use reservation::{ReservationManager, Rsvp};
use std::pin::Pin;

use abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, ConfirmRequest,
    ConfirmResponse, DbConfig, FilterRequest, FilterResponse, GetRequest, GetResponse,
    ListenRequest, ListenResponse, QueryRequest, Reservation, ReserveRequest, ReserveResponse,
    UpdateRequest, UpdateResponse,
};
use tonic::{Request, Response, Status};

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;
type ListenStream = Pin<Box<dyn Stream<Item = Result<ListenResponse, Status>> + Send>>;

pub struct RsvpService {
    manager: ReservationManager,
}

#[allow(dead_code)]
impl RsvpService {
    pub async fn from_config(config: DbConfig) -> Result<Self, Error> {
        let manager = ReservationManager::from_config(config).await?;
        Ok(Self { manager })
    }
}

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
        _request: Request<ConfirmRequest>,
    ) -> std::result::Result<Response<ConfirmResponse>, Status> {
        todo!()
    }
    async fn update(
        &self,
        _request: Request<UpdateRequest>,
    ) -> std::result::Result<Response<UpdateResponse>, Status> {
        todo!()
    }
    async fn cancel(
        &self,
        _request: Request<CancelRequest>,
    ) -> std::result::Result<Response<CancelResponse>, Status> {
        todo!()
    }
    async fn get(
        &self,
        _request: Request<GetRequest>,
    ) -> std::result::Result<Response<GetResponse>, Status> {
        todo!()
    }
    /// Server streaming response type for the query method.
    type queryStream = ReservationStream;
    async fn query(
        &self,
        _request: Request<QueryRequest>,
    ) -> std::result::Result<Response<Self::queryStream>, Status> {
        todo!()
    }
    async fn filter(
        &self,
        _request: Request<FilterRequest>,
    ) -> std::result::Result<Response<FilterResponse>, Status> {
        todo!()
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
