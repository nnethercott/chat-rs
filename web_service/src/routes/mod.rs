use crate::server::AppState;
use axum::{Router, routing::get};
use models::list_models;

mod models;

pub(crate) fn app_routes() -> Router<AppState> {
    let model_routes = Router::new().route("/list", get(list_models));
    // models/{model_id}/chat (streaming)
    // models/{model_id}/generate

    Router::new()
        .nest("/models", model_routes)
        .route("/healthz", get(async || {})) // `IntoRespone` impl for ()
}
