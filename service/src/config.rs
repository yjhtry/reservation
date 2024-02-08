use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Config {
    pub db: DbConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[allow(dead_code)]
impl Config {
    pub async fn load(filename: &str) -> Result<Self> {
        let file = fs::read_to_string(filename).await?;
        let config = serde_yaml::from_str(&file)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn load_config_should_word() {
        let config = Config::load("fixtures/config.yaml").await.unwrap();

        assert_eq!(config.db.host, "localhost");
        assert_eq!(config.db.port, 5432);
        assert_eq!(config.db.user, "yjh");
        assert_eq!(config.db.password, "");
        assert_eq!(config.db.dbname, "reservation");
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 3333);
    }
}
