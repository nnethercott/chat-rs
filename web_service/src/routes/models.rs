use axum::{extract::State, Json};
use crate::{errors::WebError, AppState};
use futures::StreamExt;

pub async fn list_models(State(client): State<AppState>) -> Result<Json<Vec<String>>, WebError> {
    let stream = client
        .lock()
        .await
        .list_models(())
        .await
        .map_err(WebError::GrpcError)?
        .into_inner();

    let models: Vec<String> = stream
        .filter_map(|i| async { i.ok() })
        .map(|spec| spec.model_id)
        .collect()
        .await;

    Ok(Json(models))
}

