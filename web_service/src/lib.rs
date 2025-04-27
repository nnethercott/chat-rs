pub mod config;
pub mod errors;
pub mod routes;

use std::{ops::Deref, sync::Arc};

use crate::config::get_config;
use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
};
use errors::WebError;
use futures::StreamExt;
use grpc_service::inferencer_client::InferencerClient;
use tokio::{net::TcpListener, sync::Mutex};
use tonic::transport::Channel;

async fn hello() -> &'static str {
    "hello, world"
}

async fn list_models(State(client): State<AppState>) -> Result<Json<Vec<String>>, WebError> {
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

#[derive(Clone)]
pub(crate) struct AppState(Arc<Mutex<InferencerClient<Channel>>>);

impl Deref for AppState{
    type Target = Mutex<InferencerClient<Channel>>;
    fn deref(&self) -> &Self::Target {
        &self.0.deref()
    }
}

pub async fn run_app() -> anyhow::Result<()> {
    let config = get_config()?;

    let grpc_client = InferencerClient::connect(format!("http://{}", config.grpc.addr())).await?;
    let state = AppState(Arc::new(Mutex::new(grpc_client)));

    let app = Router::new()
        .route("/", get(hello))
        .route("/models", get(list_models))
        .with_state(state);

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
