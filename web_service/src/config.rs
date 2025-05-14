#![allow(clippy::to_string_trait_impl)]
use clap::Parser;
use std::{
    env::{self},
    path::Path,
};

use config::Config;
use serde::Deserialize;

type RedisConfig = ServerConfig;
type GRPCConfig = ServerConfig;

#[derive(Parser, Debug, Deserialize, Default)]
pub struct Settings {
    #[serde(alias = "web", flatten)]
    #[clap(alias = "web", flatten)]
    pub server: ServerConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub redis: RedisConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub grpc: GRPCConfig,
}

#[derive(Parser, Debug, Deserialize, Default)]
pub struct ServerConfig {
    /// host for axum server
    #[clap(long, default_value = "127.0.0.1", id = "web.host")]
    pub host: String,

    /// port for axum server
    #[clap(long, default_value_t = 3000, id = "web.port")]
    pub port: u16,
}

impl ServerConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// TODO: write macro for server_config
