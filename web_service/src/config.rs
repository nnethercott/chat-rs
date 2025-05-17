#![allow(clippy::to_string_trait_impl)]
use clap::Parser;
use serde::Deserialize;

macro_rules! server_config {
    ($name:ident, $alias:literal, $host:literal, $port:literal) => {
        #[derive(Parser, Debug, Deserialize, Default)]
        pub struct $name {
            #[clap(long = stringify!($alias-host), default_value = $host, id = stringify!($alias.host))]
            pub host: String,

            #[clap(long = stringify!($alias-port), default_value_t = $port, id = stringify!($alias.port))]
            pub port: u16,
        }

        impl $name {
            pub fn addr(&self) -> String {
                format!("{}:{}", self.host, self.port)
            }
        }
    };
}

// codegen server configs
server_config!(WebConfig, "web", "127.0.0.1", 3000);
server_config!(RedisConfig, "redis", "127.0.0.1", 6379);
server_config!(GRPCConfig, "grpc", "[::1]", 50051);

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
    pub grpc: GRPCConfig,
}
