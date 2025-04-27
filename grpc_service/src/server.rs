
use crate::{
    Error, InferenceRequest, InferenceResponse, ModelSpec, ModelType,
    config::{DatabaseConfig, Settings},
    inferencer_server::{Inferencer, InferencerServer},
    pg::PgModelSpec,
};
use deadpool_postgres::{Pool, Runtime};
use tokio::{
    signal::{self, unix::SignalKind},
    sync::{Mutex, mpsc},
};
use tokio_postgres::NoTls;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, transport::Server};
use tower_http::trace::{MakeSpan, TraceLayer};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub fn generate_random_registry() -> Vec<ModelSpec> {
    (0..32)
        .map(|_| ModelSpec {
            model_id: Uuid::new_v4().to_string(),
            model_type: ModelType::Image.into(),
        })
        .collect()
}

pub async fn connect_to_db(config: &DatabaseConfig) -> Result<Pool, Error> {
    Ok(config
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(|_| anyhow::anyhow!("failed to connect to db"))?)
}

pub struct ModelServer {
    pub registry: Mutex<Vec<ModelSpec>>,
    pg_pool: Pool,
}

impl ModelServer {
    pub fn new(pg_pool: Pool) -> Self {
        ModelServer {
            pg_pool,
            registry: Mutex::new(vec![]),
        }
    }

    async fn fetch_models(&self) -> anyhow::Result<()> {
        match self.pg_pool.get().await {
            Ok(client) => {
                if let Ok(rows) = client.query("SELECT * FROM models", &[]).await {
                    let models: Vec<ModelSpec> = rows
                        .into_iter()
                        .map(|r| {
                            let wrapper: PgModelSpec = r.get(0);
                            wrapper.into()
                        })
                        .collect();
                    // update registry
                    {
                        let mut lock = self.registry.lock().await;
                        *lock = models;
                    }
                }
            }
            _ => return Err(anyhow::anyhow!("failed to get reader from pool")),
        };

        Ok(())
    }

    async fn add_models(&self, models: Vec<ModelSpec>) -> anyhow::Result<u64> {
        let values: Vec<PgModelSpec> = models.into_iter().map(PgModelSpec::from).collect();

        let n_rows = match self.pg_pool.get().await {
            Ok(client) => {
                let stmt = client
                    .prepare("INSERT INTO models(spec) SELECT unnest($1::modelspec[])")
                    .await
                    .unwrap();

                client
                    .execute(&stmt, &[&values])
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?
            }
            Err(_) => return Err(anyhow::anyhow!("failed to get reader from pool")),
        };

        // refresh model registry
        self.fetch_models().await?;
        Ok(n_rows)
    }
}

#[tonic::async_trait]
impl Inferencer for ModelServer {
    async fn run_inference(
        &self,
        _request: Request<InferenceRequest>,
    ) -> Result<Response<InferenceResponse>, Status> {
        // use onnx inference from crate we haven't defined yet ...
        todo!()
    }

    #[doc = "Server streaming response type for the ListModels method."]
    type ListModelsStream = ReceiverStream<Result<ModelSpec, Status>>;

    async fn list_models(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::ListModelsStream>, Status> {
        let (tx, rx) = mpsc::channel(4);

        let model_list = { self.registry.lock().await.clone() };

        tokio::spawn(async move {
            for spec in model_list {
                tx.send(Ok(spec)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    #[doc = "rpc runBatchedInference(stream InferenceRequest) returns (stream InferenceResponse);"]
    async fn add_models(
        &self,
        request: Request<tonic::Streaming<ModelSpec>>,
    ) -> Result<Response<u64>, Status> {
        let models: Vec<ModelSpec> = request.into_inner().filter_map(|i| i.ok()).collect().await;

        let n_rows = self.add_models(models).await.unwrap_or_else(|e| {
            error!(error=?e);
            warn!("0 models added");
            0
        });

        Ok(Response::new(n_rows))
    }
}

#[derive(Clone)]
struct ServerMakeSpan;

/// span for logging incoming requests to the server
impl<T> MakeSpan<T> for ServerMakeSpan {
    fn make_span(&mut self, request: &http::Request<T>) -> tracing::Span {
        tracing::span!(
            tracing::Level::INFO,
            "grpc request",
            method= %request.method(),
            resource = %request.uri().path(),
            span_id = %Uuid::new_v4(), // FIXME: hash this and hexdump
        )
    }
}

async fn graceful_shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.unwrap();
    };
    let sigterm = async {
        signal::unix::signal(SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm => {},
    }
}

pub async fn run_server(config: Settings) -> Result<(), Error> {
    let socket_addr = config.server.addr().parse().unwrap();

    // health
    let (reporter, health_service) = tonic_health::server::health_reporter();
    reporter
        .set_serving::<InferencerServer<ModelServer>>()
        .await;

    // reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(crate::FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .unwrap();

    // establish connection to db pool
    let pg_pool = connect_to_db(&config.db).await?;

    let model_server = ModelServer::new(pg_pool);
    model_server.fetch_models().await?;

    let server = Server::builder()
        // add tower tracing layer for requests
        .layer(TraceLayer::new_for_grpc().make_span_with(ServerMakeSpan))
        // add service layers -> [ml, reflection, health]
        .add_service(InferencerServer::new(model_server))
        .add_service(reflection_service)
        .add_service(health_service);

    let shutdown = async {
        graceful_shutdown().await;
        info!("shutting down server...");
    };

    info!("starting server...");
    debug!(config=?config);

    server
        .serve_with_shutdown(socket_addr, shutdown)
        .await
        .map_err(|e| Status::internal(format!("server failed to start: {e}")))?;

    Ok(())
}
