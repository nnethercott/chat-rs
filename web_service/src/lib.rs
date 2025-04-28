pub mod config;
pub mod errors;
pub mod routes;
use axum::{Router, routing::get};
use config::Settings;
use grpc_service::inferencer_client::InferencerClient;
use routes::app_routes;
use std::{ops::Deref, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tonic::transport::Channel;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer,
};
use tracing::Level;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct AppState(Arc<Mutex<InferencerClient<Channel>>>);

impl Deref for AppState {
    type Target = Mutex<InferencerClient<Channel>>;
    fn deref(&self) -> &Self::Target {
        &self.0.deref()
    }
}

async fn hello() -> &'static str {
    "hello, world"
}

#[derive(Clone)]
struct WebMakeSpan;

impl<T> MakeSpan<T> for WebMakeSpan {
    fn make_span(&mut self, request: &http::Request<T>) -> tracing::Span {
        tracing::span!(
            tracing::Level::INFO,
            "axum_http_request",
            method= %request.method(),
            uri = %request.uri().path(),
            span_id = %Uuid::new_v4(),
        )
    }
}

pub async fn run_app(config: Settings) -> anyhow::Result<()> {
    let grpc_client = InferencerClient::connect(format!("http://{}", config.grpc.addr())).await?;
    let state = AppState(Arc::new(Mutex::new(grpc_client)));

    let app = Router::new()
        // routes
        .merge(app_routes())
        // per-request tracing, INFO level
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(WebMakeSpan)
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Micros),
                ),
        )
    .with_state(state);

    let listener = TcpListener::bind(config.server.addr()).await.unwrap();
    let server = axum::serve(listener, app).await.unwrap();
    Ok(())
}
