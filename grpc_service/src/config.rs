#![allow(clippy::to_string_trait_impl)]

const DB_HOST: &str = "APP__DB_HOST";
const DB_PORT: &str = "APP__DB_PORT";
const DB_NAME: &str = "APP__DB_NAME";
const DB_USERNAME: &str = "APP__DB_USERNAME";
const DB_PASSWORD: &str = "APP__DB_PASSWORD";

use clap::Parser;
use serde::{Deserialize, Deserializer};
use sqlx::{PgPool, postgres::PgConnectOptions};
use std::str::FromStr;
use tracing::Level;

#[derive(Parser, Debug, Deserialize)]
pub struct Settings {
    #[serde(flatten)]
    #[clap(flatten)]
    pub server: ServerConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub db: DatabaseConfig,

    #[serde(
        default = "default_log_level",
        deserialize_with = "try_deserialize_log_level"
    )]
    #[clap(
        long,
        short='v',
        default_value_t = default_log_level(),
        value_parser=clap::value_parser!(tracing::Level)
    )]
    pub log_level: tracing::Level,
}

#[derive(Parser, Debug, Deserialize)]
pub struct DatabaseConfig {
    /// db name
    #[clap(long, env = DB_NAME, default_value = "models")]
    pub db_name: String,

    /// db username
    #[clap(long, env = DB_USERNAME, default_value = "postgres")]
    pub user_name: String,

    /// db username
    #[clap(long, env = DB_PASSWORD, default_value = "password")]
    pub password: String,

    // db host
    #[clap(long = "db-host", env = DB_HOST, default_value = "127.0.0.1", id="db.host")]
    pub host: String,

    // db port
    #[clap(long = "db-port", env = DB_PORT, default_value_t = 5432, id="db.port")]
    pub port: u16,
}

impl DatabaseConfig {
    pub fn create_pool(&self) -> PgPool {
        let opts = PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .database(&self.db_name)
            .username(&self.user_name)
            .password(&self.password);

        PgPool::connect_lazy_with(opts)
    }
}

#[derive(Parser, Deserialize, Debug)]
pub struct ServerConfig {
    // host of the grprc server
    #[clap(long = "grpc-host", default_value = "[::1]", id = "server.host")]
    pub host: String,

    // grpc port
    #[clap(long = "grpc-port", default_value = "50051", id = "server.port")]
    pub port: u16,
}

fn default_log_level() -> tracing::Level {
    Level::INFO
}

fn try_deserialize_log_level<'de, D>(d: D) -> Result<tracing::Level, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(d)
        .and_then(|v| tracing::Level::from_str(&v).map_err(serde::de::Error::custom))
}

impl ServerConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
