use axum::response::{IntoResponse, Response};
use http::StatusCode;

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
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("something went wrong: {}", self)
        ).into_response()
    }
}

