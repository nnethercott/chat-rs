use inference_service::{
    inferencer_server::{Inferencer, InferencerServer}, InferenceRequest, InferenceResponse, ModelSpec, ModelType, Null
};
use std::{fmt::Display, net::SocketAddr};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

pub mod inference_service {
    tonic::include_proto!("inferenceservice");
}
struct MLBackend {
    registry: Vec<ModelSpec>,
}
impl MLBackend {
    // for now randomly generate a list of models
    fn new() -> Self {
        let registry: Vec<ModelSpec> = (0..32)
            .map(|_| ModelSpec {
                model_id: Uuid::new_v4().to_string(),
                model_type: ModelType::Image.into(),
            })
            .collect();
        Self { registry }
    }
}

#[tonic::async_trait]
impl Inferencer for MLBackend {
    async fn run_inference(
        &self,
        request: Request<InferenceRequest>,
    ) -> Result<Response<InferenceResponse>, Status> {
        todo!()
    }

    #[doc = " Server streaming response type for the ListModels method."]
    type ListModelsStream = ReceiverStream<Result<ModelSpec, Status>>;

    async fn list_models(
        &self,
        request: Request<Null>,
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let addr: SocketAddr = "[::1]:50051".parse()?;
    let ml_service = MLBackend::new();

    let server = Server::builder()
        .add_service(InferencerServer::new(ml_service))
        .serve(addr)
        .await?;

    Ok(())
}
