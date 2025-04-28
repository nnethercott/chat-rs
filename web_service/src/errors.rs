use axum::response::{IntoResponse, Response};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GrpcStubError(#[from] tonic::Status),

    #[error(transparent)]
    GrpcConnectionError(#[from] tonic::transport::Error)
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        todo!()
    }
}

