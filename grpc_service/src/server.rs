use std::pin::Pin;

use crate::{
    Error, InferenceRequest, InferenceResponse,
    config::Settings,
    inferencer_server::{Inferencer, InferencerServer},
};
use inference_core::modelpool::{ModelPool, Opts, SendBackMessage};
use sqlx::{PgPool, QueryBuilder, Row};
use tokio::sync::{Mutex, mpsc};
use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, transport::Server};
use tower_http::trace::{MakeSpan, TraceLayer};
use tracing::{error, info, warn};
use uuid::Uuid;

// TODO: add a job that runs when new models are added to download

pub struct ModelServer {
    pub registry: Mutex<Vec<String>>,
    pub model_pool: ModelPool,
    pg_pool: PgPool,
}

impl ModelServer {
    pub fn new(pg_pool: PgPool, model_pool: ModelPool) -> anyhow::Result<Self> {
        Ok(ModelServer {
            pg_pool,
            model_pool,
            registry: Mutex::new(vec![]),
        })
    }

    async fn fetch_models(&self) -> sqlx::Result<()> {
        let models: Vec<String> = sqlx::query(r#"SELECT * FROM MODELS"#)
            .fetch_all(&self.pg_pool)
            .await?
            .iter()
            .map(|row| row.get("model_id"))
            .collect();

        *(self.registry.lock().await) = models;

        Ok(())
    }

    // batch insertion
    async fn add_models(&self, models: Vec<String>) -> sqlx::Result<u64> {
        let mut query_builder = QueryBuilder::new("INSERT INTO models(model_id)");

        // todo! maybe look into unnest
        query_builder.push_values(models, |mut b, model| {
            b.push_bind(model);
        });

        let n_rows = query_builder
            .build()
            .execute(&self.pg_pool)
            .await?
            .rows_affected();

        dbg!(n_rows);

        // refresh registry with new models
        self.fetch_models().await?;
        Ok(n_rows)
    }
}

#[tonic::async_trait]
impl Inferencer for ModelServer {
    #[doc = "Server streaming response type for the ListModels method."]
    type ListModelsStream = ReceiverStream<Result<String, Status>>;

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
        request: Request<tonic::Streaming<String>>,
    ) -> Result<Response<u64>, Status> {
        let models: Vec<String> = request.into_inner().filter_map(|i| i.ok()).collect().await;

        let n_rows = self.add_models(models).await.unwrap_or_else(|e| {
            error!(error=?e);
            warn!("0 models added");
            0
        });

        Ok(Response::new(n_rows))
    }

    #[doc = " Server streaming response type for the GenerateStreaming method."]
    type GenerateStreamingStream =
        Pin<Box<dyn Stream<Item = Result<String, Status>> + Send + Sync + 'static>>;

    async fn generate_streaming(
        &self,
        request: Request<String>,
    ) -> std::result::Result<Response<Self::GenerateStreamingStream>, Status> {
        let prompt = request.into_inner();

        let (tx, rx) = mpsc::channel(1024);
        let req = SendBackMessage::Streaming {
            prompt,
            sender: tx,
            opts: Opts::default(),
        };

        // schedule inference job
        self.model_pool.infer(req).unwrap();

        // Result<u32, Status> is a constraint from tonic; we need to adapt the rx token stream
        // into this expected format
        let adpt = ReceiverStream::new(rx).map(Ok);

        Ok(Response::new(Box::pin(adpt)))
    }

    async fn generate(
        &self,
        request: Request<InferenceRequest>,
    ) -> Result<Response<InferenceResponse>, Status> {
        let inference_request = request.into_inner();

        // this is kinda gross but we have a circular dependency if we try to import opts in
        // inference_core
        let opts: Opts = inference_request
            .opts
            .map(Into::into)
            .unwrap_or_default();

        let (tx, rx) = tokio::sync::oneshot::channel();
        let req = SendBackMessage::Blocking {
            prompt: inference_request.prompt,
            sender: tx,
            opts,
        };

        // schedule inference job
        self.model_pool.infer(req).unwrap();

        // ... and await response
        let response = rx.await.map_err(|e| Status::internal(format!("{:?}", e)))?;

        Ok(Response::new(InferenceResponse {
            response,
            ..Default::default()
        }))
    }
}

#[derive(Clone)]
struct ServerMakeSpan;

/// span for logging incoming requests to the server
impl<T> MakeSpan<T> for ServerMakeSpan {
    fn make_span(&mut self, request: &http::Request<T>) -> tracing::Span {
        let span_id: String = Uuid::new_v4().to_string().split("-").take(1).collect();

        tracing::span!(
            tracing::Level::INFO,
            "tonic_grpc_request",
            method= %request.method(),
            uri = %request.uri().path(),
            span_id = span_id
        )
    }
}

pub async fn run_server(config: Settings, model_pool: ModelPool) -> Result<(), Error> {
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

    // connect to db and refresh models
    let model_server = ModelServer::new(config.db.create_pool(), model_pool)?;
    if let Err(e) = model_server.fetch_models().await {
        error!(error=%e);
    }

    let server = Server::builder()
        // add tower tracing layer for requests
        .layer(TraceLayer::new_for_grpc().make_span_with(ServerMakeSpan))
        // add service layers -> [ml, reflection, health]
        .add_service(InferencerServer::new(model_server))
        .add_service(reflection_service)
        .add_service(health_service);

    info!("starting server...");
    // info!(config=?config);

    server
        .serve(socket_addr)
        .await
        .map_err(|e| Status::internal(format!("server failed to start: {e}")))?;

    Ok(())
}
