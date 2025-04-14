#![allow(unused_variables, dead_code)]
use crate::{
    InferenceRequest, InferenceResponse, ModelSpec, Null,
    inferencer_server::{Inferencer, InferencerServer},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

pub struct ModelServer {
    pub registry: Vec<ModelSpec>,
}
impl ModelServer {
    pub fn new() -> Self {
        Self { registry: vec![] }
    }
}

mod inference_service {
    tonic::include_proto!("inferenceservice");
}

#[tonic::async_trait]
impl Inferencer for ModelServer {
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
