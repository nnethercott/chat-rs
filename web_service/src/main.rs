use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
};
use futures::{StreamExt, lock::Mutex};
use grpc_service::inferencer_client::InferencerClient;
use tokio::net::TcpListener;
use tonic::transport::Channel;
use web_service::config::get_config;

async fn hello() -> &'static str {
    "hello, world"
}

#[derive(thiserror::Error, Debug)]
enum WebError {
    #[error(transparent)]
    GrpcError(tonic::Status),
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        todo!()
    }
}

async fn list_models(
    State(client): State<Arc<Mutex<InferencerClient<Channel>>>>,
) -> Result<Json<Vec<String>>, WebError> {
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = get_config()?;

    let grpc_client = InferencerClient::connect(format!("http://{}", config.grpc.addr())).await?;

    let app = Router::new()
        .route("/", get(hello))
        .route("/models", get(list_models))
        .with_state(Arc::new(Mutex::new(grpc_client)));

    let listener = TcpListener::bind(config.server.addr()).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

// gonna store these in a redis db per user session id
// #[derive(Serialize, Deserialize)]
// struct ChatHistory {
//     messages: Vec<Message>,
// }
//
// #[derive(Serialize, Deserialize)]
// struct Message {
//     role: Role,
//     content: String,
// }
//
// #[derive(Serialize, Deserialize)]
// enum Role {
//     User,
//     Assistant,
// }
