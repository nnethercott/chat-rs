use crate::{Result, server::AppState};
use axum::{Json, extract::State};
use futures::StreamExt;
use grpc_service::ModelSpec;

#[axum::debug_handler]
pub async fn list_models(State(state): State<AppState>) -> Result<Json<Vec<ModelSpec>>> {
    match state.client() {
        Some(client) => {
            let stream = client.lock().await.list_models(()).await?.into_inner();

            let models: Vec<ModelSpec> = stream.filter_map(|i| async { i.ok() }).collect().await;

            Ok(Json(models))
        }
        None => Ok(Json(vec![])),
    }
}
