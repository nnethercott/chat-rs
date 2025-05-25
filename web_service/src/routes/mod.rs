use crate::server::AppState;
use axum::{Router, response::IntoResponse, routing::get};

mod chat;
mod completion;
mod models;

use chat::chat;
// use completion::completions;
use models::list_models;
use tower_sessions::Session;

pub async fn count(sesh: Session) -> impl IntoResponse {
    let mut val: usize = sesh.get("NATE").await.unwrap().unwrap_or_default();
    sesh.insert("NATE", val+1).await.unwrap();
    format!("count is {}", val)
}

pub(crate) fn app_routes() -> Router<AppState> {
    let model_routes = Router::new()
        .route("/list", get(list_models))
        .route("/{id}/chat", get(chat));
    // .route("/{id}/completions", post(completions));

    Router::new()
        .nest("/models", model_routes)
        .route("/healthz", get(async || {})) // `IntoRespone` impl for ()
        .route("/count", get(count))
}
