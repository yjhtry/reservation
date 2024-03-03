use abi::{Config, ListenResponse, Reservation};
use anyhow::Error;
use futures::stream::Stream;
use reservation::ReservationManager;
use std::{net::SocketAddr, pin::Pin, task::Poll};
use tokio::sync::mpsc;
use tonic::Status;

mod service;

#[cfg(test)]
pub mod test_util;

pub type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;
pub type ListenStream = Pin<Box<dyn Stream<Item = Result<ListenResponse, Status>> + Send>>;

pub struct RsvpService {
    pub manager: ReservationManager,
}

impl RsvpService {
    pub async fn from_config(config: &Config) -> Result<Self, Error> {
        let manager = ReservationManager::from_config(&config.db).await?;
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

pub async fn start_server(config: &Config) -> Result<(), Error> {
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    let svc = RsvpService::from_config(config).await?;
    let svc = abi::reservation_service_server::ReservationServiceServer::new(svc);

    println!("Listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;
    Ok(())
}
