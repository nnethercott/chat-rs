use config::ConfigError;
use thiserror;
use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PgConnectionError(#[from] anyhow::Error),

    #[error("something went wrong in server init!\n{0}")]
    ServerSpawnError(#[from] Status),
}

