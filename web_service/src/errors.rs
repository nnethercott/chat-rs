use axum::response::{IntoResponse, Response};

#[derive(thiserror::Error, Debug)]
pub enum WebError {
    #[error(transparent)]
    GrpcError(tonic::Status),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        todo!()
    }
}

