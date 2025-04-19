use std::{
    env::{self},
    str::FromStr,
};

use config::Config;
use serde::{Deserialize, Deserializer};
use tracing::Level;

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

impl Settings {
    pub fn new() -> Self {
        todo!();
    }
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

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub pg: deadpool_postgres::Config,
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

    let c = {
        let config_file = env::current_dir()
            .unwrap()
            .join(format!("grpc_service/config/{}.yaml", environ.to_string()));

        Config::builder()
            .add_source(config::File::from(config_file))
            .add_source(config::Environment::default().separator("__"))
            .build()?
    };

    c.try_deserialize()
}
