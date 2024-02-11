use abi::{Config, ListenResponse, Reservation};
use anyhow::Error;
use futures::stream::Stream;
use reservation::ReservationManager;
use std::{pin::Pin, task::Poll};
use tokio::sync::mpsc;
use tonic::Status;

mod service;

mod test_util;

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
