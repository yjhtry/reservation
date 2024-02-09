use anyhow::Error;
use futures::stream::Stream;
use reservation::{ReservationManager, Rsvp};
use std::{pin::Pin, task::Poll};
use tokio::sync::mpsc;

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

pub struct TonicReceiverStream<T> {
    inner: mpsc::Receiver<Result<T, abi::Error>>,
}

impl<T> TonicReceiverStream<T> {
    pub fn new(inner: mpsc::Receiver<Result<T, abi::Error>>) -> Self {
        Self { inner }
    }
}

impl<T> Stream for TonicReceiverStream<T> {
    type Item = Result<T, Status>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.inner.poll_recv(cx) {
            Poll::Ready(Some(Ok(item))) => Poll::Ready(Some(Ok(item))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
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
