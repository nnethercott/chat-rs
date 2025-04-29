use crate::{Result, server::AppState};
use axum::{Json, extract::State};
use futures::StreamExt;

pub async fn list_models(State(state): State<AppState>) -> Result<Json<Vec<String>>> {
    match state.client() {
        Some(client) => {
            let stream = client.lock().await.list_models(()).await?.into_inner();

            let models: Vec<String> = stream
                .filter_map(|i| async { i.ok() })
                .map(|spec| spec.model_id)
                .collect()
                .await;

            Ok(Json(models))
        },
        None =>  todo!()
    }
}
