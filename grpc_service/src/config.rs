#![allow(clippy::to_string_trait_impl)]

use std::{
    env::{self},
    path::Path,
    str::FromStr,
};

use config::Config;
use serde::{Deserialize, Deserializer};
use sqlx::{PgPool, postgres::PgConnectOptions};
use tracing::Level;

// implements deserialize already
#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub db_name: String,
    pub user_name: String,
    pub password: String,
    pub host: String,
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

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub server: ServerConfig,
    pub db: DatabaseConfig,
    #[serde(
        default = "default_log_level",
        deserialize_with = "try_deserialize_loglevel"
    )]
    pub log_level: tracing::Level,
}

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
}

fn default_log_level() -> Level {
    Level::INFO
}

fn try_deserialize_loglevel<'de, D>(d: D) -> Result<tracing::Level, D::Error>
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

pub enum Env {
    Local,
    Production,
}

impl ToString for Env {
    fn to_string(&self) -> String {
        match self {
            Env::Local => "local".into(),
            Env::Production => "production".into(),
        }
    }
}

impl TryFrom<String> for Env {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &value[..] {
            "local" => Ok(Env::Local),
            "production" => Ok(Env::Production),
            _ => Err("value must be 'local' or 'production'".to_string()),
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let environ: Env = env::var("GRPC_SERVER_APP_ENV")
        .unwrap_or("local".into())
        .try_into()
        .expect("failed TryInto<Env>");

    let config_file = match env::var("CONFIG_FILE") {
        Ok(f) => Path::new(&f).to_path_buf(),
        Err(_) => env::current_dir().unwrap().join(format!(
            "{}/config/{}.yaml",
            env!("CARGO_CRATE_NAME"),
            environ.to_string()
        )),
    };

    let c = Config::builder()
        .add_source(config::File::from(config_file))
        .add_source(
            config::Environment::default()
                .prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    c.try_deserialize()
}
