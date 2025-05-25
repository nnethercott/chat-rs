#![allow(clippy::to_string_trait_impl)]
use axum::extract::connect_info::Connected;
use clap::Parser;
use grpc_service::config::ServerConfig as GrpcConfig;
use serde::Deserialize;
use tower_sessions_redis_store::{
    RedisStore,
    fred::{prelude::*, types::ConnectHandle},
};

#[derive(Parser, Debug, Deserialize, Default)]
pub struct WebConfig {
    /// web host
    #[clap(long = "web-host", default_value = "127.0.0.1", id = "web.host")]
    pub host: String,

    /// web port
    #[clap(long = "web-port", default_value_t = 3000, id = "web.port")]
    pub port: u16,
}

impl WebConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Parser, Debug, Deserialize, Default)]
pub struct RedisConfig {
    /// Redis host
    #[clap(long = "redis-host", default_value = "0.0.0.0", id = "redis.host")]
    pub host: String,

    /// Redis port
    #[clap(long = "redis-port", default_value_t = 6379, id = "redis.port")]
    pub port: u16,

    /// Redis password
    #[clap(long = "redis-password", default_value = "password")]
    pub password: String,
}

impl RedisConfig {
    pub async fn connect(&self) -> FredResult<(RedisStore<Pool>, ConnectHandle)> {
        let conn_str = format!(
            "redis://default:{}@{}:{}/0",
            &self.password, &self.host, &self.port,
        );
        let fred_config = Config::from_url(&conn_str)?;
        let pool = Pool::new(fred_config, None, None, None, 2)?;
        let redis_conn = pool.connect();
        pool.wait_for_connect().await?;
        Ok((RedisStore::new(pool), redis_conn))
    }
}

#[derive(Parser, Debug, Deserialize, Default)]
pub struct Settings {
    #[serde(flatten)]
    #[clap(flatten)]
    pub server: WebConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub redis: RedisConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub grpc: GrpcConfig,
}
