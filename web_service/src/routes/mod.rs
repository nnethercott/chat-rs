use crate::{
    Result,
    messages::{Messages, MessagesData},
    server::AppState,
};
use axum::{
    Json, Router,
    response::{IntoResponse, Redirect},
    routing::get,
};

mod chat;
mod models;

use chat::chat;
use grpc_service::{Role, Turn};
use http::StatusCode;
use models::list_models;
use tower_sessions::Session;
use tracing::info;

// test route
pub async fn messages(messages: Messages) -> impl IntoResponse {
    info!(session_id=?messages.session.id());
    Json(messages.data) // Return JSON instead of Debug format
}

pub(crate) fn app_routes() -> Router<AppState> {
    let model_routes = Router::new()
        .route("/list", get(list_models))
        .route("/{id}/chat", get(chat));

    Router::new()
        .nest("/models", model_routes)
        .route("/healthz", get(async || {})) // `IntoRespone` impl for ()
        .route("/messages", get(messages))
}
