use reservation_service::RsvpService;
use std::{net::SocketAddr, path::Path};

use abi::Config;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let filename = std::env::var("RESERVATIONS_CONFIG").unwrap_or_else(|_| {
        let p1 = Path::new("./reservation.yaml");
        let path = shellexpand::tilde("~/.config/reservation.yaml").to_string();
        let p2 = Path::new(&path);
        let p3 = Path::new("/etc/reservation.yaml");

        match (p1.exists(), p2.exists(), p3.exists()) {
            (true, _, _) => p1.to_str().unwrap().to_string(),
            (_, true, _) => p2.to_str().unwrap().to_string(),
            (_, _, true) => p3.to_str().unwrap().to_string(),
            _ => panic!("No configuration file found"),
        }
    });

    let config = Config::load(&filename)?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    let svc = RsvpService::from_config(&config).await?;
    let svc = abi::reservation_service_server::ReservationServiceServer::new(svc);

    println!("Listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;
    Ok(())
}
