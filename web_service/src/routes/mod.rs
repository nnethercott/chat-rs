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

pub async fn add_dummy_message(mut messages: Messages) -> Result<()> {
    messages.push_msg(Role::User.into(), "nate is cool");
    messages.update_session().await
}

// hack: init the session since we directly upgrade to ws in /chat
pub async fn _init_session(session: Session) -> impl IntoResponse {
    session
        .insert("init", "OK")
        .await
        .map_err(|_| ("failed to init", StatusCode::INTERNAL_SERVER_ERROR));

    Redirect::permanent("/chat")
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
