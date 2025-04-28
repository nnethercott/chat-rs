use axum::{Router, routing::get};
use models::list_models;

use crate::AppState;

mod chat;
mod embed;
mod models;

async fn hello() -> &'static str {
    "hello, world"
}

pub fn app_routes() -> Router<AppState> {
    let model_routes = Router::new()
        .route("/list", get(list_models));
        // .route("/{id}/chat", todo!()) // not married to this routing ... 
        // .route("/{id}/embed", todo!());

    Router::new()
        .nest("/models", model_routes)
        .route("/hello", get(hello))
}
