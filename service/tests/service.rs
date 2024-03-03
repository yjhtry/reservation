#[path = "../src/test_util.rs"]
mod test_utils;

use std::{sync::Once, time::Duration};
use tokio::time;

use abi::{
    Config, ConfirmRequest, FilterRequest, FilterResponse, QueryRequest, Reservation,
    ReservationFilterBuilder, ReservationQueryBuilder, ReservationStatus, ReserveRequest,
};

use test_utils::TestConfig;

use reservation_service::start_server;

static START: Once = Once::new();

#[tokio::test]
async fn grpc_server_should_work() {
    let config = TestConfig::with_server_port(50000);
    let config_clone = config.clone();
    start_service(config_clone).await;

    let mut client = abi::reservation_service_client::ReservationServiceClient::connect(
        config.server.url(false),
    )
    .await
    .unwrap();

    let rsvp = Reservation::new_pending(
        "john",
        "room_01",
        "2022-12-26T15:00:00-0700".parse().unwrap(),
        "2022-12-30T12:00:00-0700".parse().unwrap(),
        "I need this room for a meeting",
    );

    let request = tonic::Request::new(ReserveRequest::new(rsvp.clone()));

    let ret = client
        .reserve(request)
        .await
        .unwrap()
        .into_inner()
        .reservation
        .unwrap();

    assert_eq!(ret, rsvp);

    // test conflict reservation
    let rsvp = Reservation::new_pending(
        "john",
        "room_01",
        "2022-12-28T15:00:00-0700".parse().unwrap(),
        "2022-12-30T12:00:00-0700".parse().unwrap(),
        "I need this room for a meeting",
    );

    let request = tonic::Request::new(ReserveRequest::new(rsvp.clone()));

    let ret2 = client.reserve(request).await;

    assert!(ret2.is_err());

    // test confirm reservation
    let request = tonic::Request::new(ConfirmRequest::new(ret.id));

    let ret = client
        .confirm(request)
        .await
        .unwrap()
        .into_inner()
        .reservation
        .unwrap();

    assert_eq!(ret.status, ReservationStatus::Confirmed as i32);

    // insert 100 reservations
    for i in 0..100 {
        let rsvp = Reservation::new_pending(
            "john",
            format!("house_{}", i),
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "I need this room for a meeting",
        );

        let request = tonic::Request::new(ReserveRequest::new(rsvp.clone()));

        client
            .reserve(request)
            .await
            .unwrap()
            .into_inner()
            .reservation
            .unwrap();
    }

    let filter = ReservationFilterBuilder::default()
        .user_id("john")
        .build()
        .unwrap();

    let request = tonic::Request::new(FilterRequest::new(filter));

    let FilterResponse {
        pager,
        reservations: _,
    } = client.filter(request).await.unwrap().into_inner();

    let pager = pager.unwrap();
    // let reservations = reservations;

    assert_eq!(pager.total, 101);
    assert_eq!(pager.prev, 3);
    assert_eq!(pager.next, 12);
}

#[tokio::test]
async fn grpc_query_should_work() {
    let config = TestConfig::with_server_port(50001);
    let config_clone = config.clone();
    start_service(config_clone).await;

    let mut client = abi::reservation_service_client::ReservationServiceClient::connect(
        config.server.url(false),
    )
    .await
    .unwrap();

    // insert 100 reservations
    for i in 0..100 {
        let rsvp = Reservation::new_pending(
            "john",
            format!("house_{}", i),
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "I need this room for a meeting",
        );

        let request = tonic::Request::new(ReserveRequest::new(rsvp.clone()));

        client
            .reserve(request)
            .await
            .unwrap()
            .into_inner()
            .reservation
            .unwrap();
    }

    let query = ReservationQueryBuilder::default()
        .user_id("john")
        .build()
        .unwrap();

    let request = tonic::Request::new(QueryRequest::new(query));

    let mut ret = client.query(request).await.unwrap().into_inner();

    while let Some(reservation) = ret.message().await.unwrap() {
        assert_eq!(reservation.user_id, "john");
    }
}

async fn start_service(config: Config) {
    START.call_once(|| {
        tokio::spawn(async move {
            start_server(&config).await.unwrap();
        });
    });

    time::sleep(Duration::from_millis(500)).await;
}
