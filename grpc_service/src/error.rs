use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PgConnectionError(#[from] tokio_postgres::Error),

    #[error("something went wrong in server init!\n{0}")]
    ServerSpawnError(#[from] tonic::Status),
}
