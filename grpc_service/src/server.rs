#![allow(unused_variables, dead_code)]

use crate::{
    InferenceRequest, InferenceResponse, ModelSpec,
    configuration::{Settings, generate_random_registry},
    inferencer_server::{Inferencer, InferencerServer},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, transport::Server};

pub struct ModelServer {
    pub registry: Vec<ModelSpec>,
}
impl ModelServer {
    pub fn new() -> Self {
        Self { registry: vec![] }
    }
}

#[tonic::async_trait]
impl Inferencer for ModelServer {
    async fn run_inference(
        &self,
        request: Request<InferenceRequest>,
    ) -> Result<Response<InferenceResponse>, Status> {
        // use onnx inference from crate we haven't defined yet ...
        todo!()
    }

    #[doc = "Server streaming response type for the ListModels method."]
    type ListModelsStream = ReceiverStream<Result<ModelSpec, Status>>;

    async fn list_models(
        &self,
        request: Request<()>,
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

pub async fn run_server(config: Settings) -> tonic::Result<()> {
    let addr = "[::1]:50051";
    let socket_addr = addr.parse().unwrap();

    // FIXME: add db connection string to new() ?
    // or better use a builder pattern -> ModelServer::builder().connect_registry(db_string)?;
    // with custom db connection error
    let mut ml_service = ModelServer::new();
    ml_service.registry = generate_random_registry();

    //health check rpc
    let (reporter, health_service) = tonic_health::server::health_reporter();
    reporter
        .set_serving::<InferencerServer<ModelServer>>()
        .await;

    // reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(crate::FILE_DESCRIPTOR_SET)
        .build_v1alpha()
        .unwrap();

    Server::builder()
        .add_service(InferencerServer::new(ml_service))
        .add_service(reflection_service)
        .add_service(health_service)
        .serve(socket_addr)
        .await
        .unwrap();

    Ok(())
}
