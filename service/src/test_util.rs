use abi::Config;
use sqlx::{Connection, Executor, PgConnection};
use std::{ops::Deref, sync::Arc, thread};
use tokio::runtime::Runtime;
use uuid::Uuid;

pub struct TestConfig {
    pub config: Arc<Config>,
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
            let rt = Runtime::new().unwrap();

            rt.block_on(async move {
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

impl Default for TestConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestConfig {
    fn drop(&mut self) {
        let server_url = self.db.server_url();
        let dbname = self.config.db.dbname.clone();

        thread::spawn(move || {
          let rt = Runtime::new().unwrap();
            rt.block_on(async move {
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
