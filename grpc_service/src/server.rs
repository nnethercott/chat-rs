use crate::{
    Error as CrateError, InferenceRequest, InferenceResponse, ModelSpec, ModelType,
    config::{DatabaseConfig, Settings},
    inferencer_server::{Inferencer, InferencerServer},
};
use deadpool_postgres::{Pool, Runtime};
use tokio::sync::mpsc;
use tokio_postgres::NoTls;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, transport::Server};
use tower_http::trace::{MakeSpan, TraceLayer};
use tracing::warn;
use uuid::Uuid;

pub fn generate_random_registry() -> Vec<ModelSpec> {
    (0..32)
        .map(|_| ModelSpec {
            model_id: Uuid::new_v4().to_string(),
            model_type: ModelType::Image.into(),
        })
        .collect()
}

pub async fn connect_to_db(config: &DatabaseConfig) -> Result<Pool, CrateError> {
    Ok(config
        .pg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(|_| anyhow::anyhow!("failed to connect to db"))?)
}

// TODO: check out Dust's code to implement something similar
// impl FromSql for ModelSpec {
//     fn from_sql(
//         ty: &tokio_postgres::types::Type,
//         raw: &'a [u8],
//     ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
//         todo!()
//     }
//
//     fn accepts(ty: &tokio_postgres::types::Type) -> bool {
//         todo!()
//     }
// }

// TODO: maybe define trait for BackendServer or something ...
pub struct ModelServer {
    pub registry: Vec<ModelSpec>,
    pg_pool: Pool,
}

impl ModelServer {
    pub fn new(pg_pool: Pool) -> Self {
        ModelServer {
            pg_pool,
            registry: vec![],
        }
    }

    async fn fetch_models(&mut self) -> Result<(), CrateError> {
        match self.pg_pool.get().await {
            Ok(client) => {
                // disgusting
                if let Ok(rows) = client.query("SELECT * FROM models", &[]).await {
                    let models: Vec<ModelSpec> = rows
                        .into_iter()
                        .map(|r| {
                            let model_id: String = r.get::<usize, String>(0);
                            let model_type: i32 = r.get::<usize, i32>(1);
                            ModelSpec {
                                model_id,
                                model_type,
                            }
                        })
                        .collect();
                    self.registry = models;
                }
            }
            Err(_) => warn!("failed to get reader from pool"),
        }
        Ok(())
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

pub async fn run_server(config: Settings, pg_pool: Pool) -> Result<(), CrateError> {
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

    let mut model_server = ModelServer::new(pg_pool);
    model_server.fetch_models().await?;

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
