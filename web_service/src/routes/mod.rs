use crate::server::AppState;
use axum::{Router, routing::get};
use models::list_models;

mod models;

pub(crate) fn app_routes() -> Router<AppState> {
    let model_routes = Router::new().route("/list", get(list_models));

    Router::new()
        .nest("/models", model_routes)
        .route("/health", get(async || {})) // `IntoRespone` impl for ()
}
