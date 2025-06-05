use crate::routes::{_init_session, add_dummy_message};
use crate::Result;
use crate::{config::Settings, routes::app_routes};
use axum::Router;
use axum::response::Html;
use axum::routing::get;
use grpc_service::inferencer_client::InferencerClient;
use std::sync::Arc;
use tokio::signal;
use tokio::{net::TcpListener, sync::Mutex};
use tonic::transport::Channel;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer, session};
use tracing::{Level, debug, error, info, warn};
use uuid::Uuid;

type Inner = InferencerClient<Channel>;

// TODO: maybe generalize this to ANY grpc service with generics
#[derive(Clone, Default)]
pub struct AppState {
    inner: Option<Arc<Mutex<Inner>>>,
}

impl AppState {
    pub fn new(client: Inner) -> Self {
        Self {
            inner: Some(Arc::new(Mutex::new(client))),
        }
    }

    // since deref won't work here
    pub fn client(&self) -> Option<&Mutex<Inner>> {
        self.inner.as_deref()
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
    pub fn new(config: Settings) -> Result<Self> {
        // /!\ Add state in App::run() otherwise won't compile;
        // https://docs.rs/axum/latest/axum/struct.Router.html#method.with_state
        let app = Router::new()
            // routes
            .merge(app_routes())
            .route("/", get(_init_session))
            .route("/dummy", get(add_dummy_message))
            .route("/chat", get(async || Html(include_str!("../chat.html"))))
            // request tracing
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(WebMakeSpan)
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(tower_http::LatencyUnit::Millis),
                    ),
            );

        Ok(Self { app, config })
    }

    /// opt into persistent sessions
    /// NOTE: can also configure redis to connect to postgres for read-through behaviour
    pub async fn new_with_session_store(config: Settings) -> Result<Self> {
        let App { app, config } = Self::new(config)?;

        match &config.redis {
            Some(redis) => {
                info!("redis config found");
                let (session_store, _) = redis.connect().await?;
                let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

                let app = app.layer(session_layer);
                Ok(App { app, config })
            }
            // no-op
            None => {
                warn!("redis config not found, using memory store");
                let session_store = MemoryStore::default();
                let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

                let app = app.layer(session_layer);
                Ok(App { app, config })
            }
        }
    }

    // TODO: add grpc client pool
    pub async fn run(self) -> Result<()> {
        // connect to grpc service
        let inference_client =
            InferencerClient::connect(format!("http://{}", self.config.grpc.addr())).await?;
        let state = AppState::new(inference_client);

        // bind to tcp port
        let listener = TcpListener::bind(self.config.server.addr())
            .await
            .expect("port in use");

        info!("Starting web server...");
        if let Err(e) = axum::serve(listener, self.app.with_state(state))
            .with_graceful_shutdown(shutdown())
            .await
        {
            error!(error=?e);
            info!("Shutting down...")
        }
        Ok(())
    }

    pub fn into_router(self) -> Router<AppState> {
        self.app
    }
}

async fn shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("[GRACEFUL] shutdown with ctrl-c...")
        },
        _ = terminate => {
            info!("[GRACEFUL] shutdown on kill sig...")
        },
    }
}
