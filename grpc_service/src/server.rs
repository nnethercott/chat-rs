use crate::{
    Error, InferenceRequest, InferenceResponse, ModelSpec, ModelType,
    config::Settings,
    inferencer_server::{Inferencer, InferencerServer},
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_postgres::Client;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, transport::Server};
use tower_http::trace::{MakeSpan, TraceLayer};
use uuid::Uuid;

pub fn generate_random_registry() -> Vec<ModelSpec> {
    (0..32)
        .map(|_| ModelSpec {
            model_id: Uuid::new_v4().to_string(),
            model_type: ModelType::Image.into(),
        })
        .collect()
}

pub struct ModelServer {
    pub registry: Vec<ModelSpec>,
    pub pg_client: Arc<Client>,
}
impl ModelServer {
    pub fn new(pg_client: Client) -> Self {
        Self {
            pg_client: Arc::new(pg_client),
            registry: vec![],
        }
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

        let model_list = self.registry.clone();
        tokio::spawn(async move {
            for spec in model_list {
                tx.send(Ok(spec)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[derive(Clone)]
struct ServerMakeSpan;

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

pub async fn run_server(config: Settings, pg_client: tokio_postgres::Client) -> Result<(), Error> {
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

    let model_server = ModelServer::new(pg_client);

    Server::builder()
        // add tracing layer
        .layer(TraceLayer::new_for_grpc().make_span_with(ServerMakeSpan))
        // add service layers -> [ml, reflection, health]
        .add_service(InferencerServer::new(model_server))
        .add_service(reflection_service)
        .add_service(health_service)
        .serve(socket_addr)
        .await
        .map_err(|e| Status::internal(format!("server failed to start: {e}")))?;

    Ok(())
}
