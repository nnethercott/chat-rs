use crate::Result;
use crate::{config::Settings, routes::app_routes};
use axum::Router;
use grpc_service::inferencer_client::InferencerClient;
use std::{ops::Deref, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tonic::transport::Channel;
use tower_http::trace::{
    DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer,
};
use tracing::{Level, error, info};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct AppState(Arc<Mutex<InferencerClient<Channel>>>);

impl Deref for AppState {
    type Target = Mutex<InferencerClient<Channel>>;
    fn deref(&self) -> &Self::Target {
        &self.0.deref()
    }
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

/// public API to our web server
pub struct App {
    app: Router<AppState>,
    config: Settings,
}

impl App {
    pub fn new(
        config: Settings,
    ) -> Result<Self> {
        // /!\ Add state in App::run() otherwise won't compile;
        // https://docs.rs/axum/latest/axum/struct.Router.html#method.with_state
        let app = Router::new()
            // routes
            .merge(app_routes())
            // tracing
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(WebMakeSpan)
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(tower_http::LatencyUnit::Micros),
                    ),
            );

        Ok(Self { app, config })
    }

    // TODO: add graceful shutdown
    pub async fn run(self) -> Result<()> {
        // connect to grpc service
        let inference_client =
            InferencerClient::connect(format!("http://{}", self.config.grpc.addr())).await?;
        let state = AppState(Arc::new(Mutex::new(inference_client)));

        // bind to tcp port
        let listener = TcpListener::bind(self.config.server.addr()).await.expect("port in use");

        info!("starging web server...");
        if let Err(e) = axum::serve(listener, self.app.with_state(state)).await {
            error!(error=?e);
        }
        Ok(())
    }
}
