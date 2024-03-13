#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let mut listener = sqlx::postgres::PgListener::connect(&url).await.unwrap();

    listener.listen("reservation_update").await.unwrap();

    println!("Listening for notifications...");

    loop {
        let notification = listener.recv().await.unwrap();
        println!("Received notification: {:?}", notification);
    }
}
