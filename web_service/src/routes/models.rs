use axum::{extract::State, Json};
use crate::{Result, server::AppState};
use futures::StreamExt;

pub async fn list_models(State(client): State<AppState>) -> Result<Json<Vec<String>>> {
    let stream = client
        .lock()
        .await
        .list_models(())
        .await?
        .into_inner();

    let models: Vec<String> = stream
        .filter_map(|i| async { i.ok() })
        .map(|spec| spec.model_id)
        .collect()
        .await;

    Ok(Json(models))
}

