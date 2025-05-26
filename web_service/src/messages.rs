use axum::{
    extract::FromRequestParts,
    response::{IntoResponseParts, ResponseParts},
};
use grpc_service::Turn;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

#[derive(Serialize, Deserialize)]
pub struct Messages(Vec<Turn>);

impl<S> FromRequestParts<S> for Messages
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        todo!()
    }
}
