use crate::server::AppState;
use axum::{Router, routing::get};
use models::list_models;

mod chat;
mod embed;
mod models;

// `IntoResponse` implemented for ()
async fn health() {}

pub(crate) fn app_routes() -> Router<AppState> {
    let model_routes = Router::new().route("/list", get(list_models));
    // .route("/{id}/chat", todo!()) // not married to this routing ...
    // .route("/{id}/embed", todo!());

    Router::new()
        .nest("/models", model_routes)
        .route("/health", get(health))
}
