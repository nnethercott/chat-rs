use crate::server::AppState;
use axum::{Router, routing::get};

mod chat;
mod models;

use chat::chat;
use models::list_models;

pub(crate) fn app_routes() -> Router<AppState> {
    let model_routes = Router::new()
        .route("/list", get(list_models))
        .route("/{id}/chat", get(chat));

    // models/{model_id}/generate

    Router::new()
        .nest("/models", model_routes)
        .route("/healthz", get(async || {})) // `IntoRespone` impl for ()
}
