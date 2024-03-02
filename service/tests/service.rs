use std::time::Duration;
use tokio::time;

use abi::{
    ConfirmRequest, FilterRequest, FilterResponse, Reservation, ReservationFilterBuilder,
    ReservationStatus, ReserveRequest,
};

use reservation_service::{start_server, test_util::TestConfig};

#[tokio::test]
async fn grpc_server_should_work() {
    let config = TestConfig::new();

    let config_clone = config.clone();

    tokio::spawn(async move {
        let _ = start_server(&config_clone).await;
    });

    time::sleep(Duration::from_millis(200)).await;

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
