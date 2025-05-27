use crate::{messages::Messages, server::AppState};
use axum::{Router, response::IntoResponse, routing::get};

mod chat;
mod completion;
mod models;

use chat::chat;
// use completion::completions;
use models::list_models;
use tower_sessions::Session;

// test route
pub async fn messages(messages: Messages) -> impl IntoResponse {
    messages.to_string() 
}

pub(crate) fn app_routes() -> Router<AppState> {
    let model_routes = Router::new()
        .route("/list", get(list_models))
        .route("/{id}/chat", get(chat));
    // .route("/{id}/completions", post(completions));

    Router::new()
        .nest("/models", model_routes)
        .route("/healthz", get(async || {})) // `IntoRespone` impl for ()
        .route("/messages", get(messages))
}
