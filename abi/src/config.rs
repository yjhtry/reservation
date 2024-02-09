use std::fs;

use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Config {
    pub db: DbConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,

    #[serde(default = "default_max_connects")]
    pub max_connects: u32,
}

impl DbConfig {
    pub fn url(&self) -> String {
        if self.password.is_empty() {
            format!(
                "postgres://{}@{}:{}/{}",
                self.user, self.host, self.port, self.dbname
            )
        } else {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.user, self.password, self.host, self.port, self.dbname
            )
        }
    }
}

fn default_max_connects() -> u32 {
    5
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[allow(dead_code)]
impl Config {
    pub fn load(filename: &str) -> Result<Config, Error> {
        let file = fs::read_to_string(filename).map_err(|_| Error::ReadConfigError)?;
        let config = serde_yaml::from_str(&file).map_err(|_| Error::ParseConfigError)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_config_should_word() {
        let config = Config::load("../service/fixtures/config.yaml").unwrap();

        assert_eq!(config.db.host, "localhost");
        assert_eq!(config.db.port, 5432);
        assert_eq!(config.db.user, "yjh");
        assert_eq!(config.db.password, "");
        assert_eq!(config.db.dbname, "reservation");
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 3333);
    }
}
