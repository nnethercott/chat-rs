use std::pin::Pin;

use futures::Stream;
use grpc_service::{
    InferenceRequest, InferenceResponse,
    inferencer_client::InferencerClient,
    inferencer_server::{Inferencer, InferencerServer},
};
use http::Uri;
use hyper_util::rt::TokioIo;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
    Request, Response, Status, Streaming,
    transport::{Channel, Endpoint, Server},
};
use tower::service_fn;

pub struct MockGrpc;

// a whole new grpc server ??
#[tonic::async_trait]
impl Inferencer for MockGrpc {
    type ListModelsStream = ReceiverStream<Result<String, Status>>;

    async fn list_models(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::ListModelsStream>, Status> {
        let (tx, rx) = mpsc::channel(4);

        let model_list = vec!["model1".into(), "model2".into(), "model3".into()];

        tokio::spawn(async move {
            for spec in model_list {
                tx.send(Ok(spec)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn add_models(
        &self,
        _request: tonic::Request<Streaming<String>>,
    ) -> Result<Response<u64>, Status> {
        Ok(Response::new(42))
    }

    #[doc = " Server streaming response type for the GenerateStreaming method."]
    type GenerateStreamingStream =
        Pin<Box<dyn Stream<Item = Result<String, Status>> + Send + Sync + 'static>>;

    async fn generate_streaming(
        &self,
        _request: tonic::Request<String>,
    ) -> std::result::Result<Response<Self::GenerateStreamingStream>, Status> {
        todo!()
    }

    async fn generate(
        &self,
        request: Request<InferenceRequest>,
    ) -> Result<Response<InferenceResponse>, Status> {
        todo!()
    }
}

// https://github.com/hyperium/tonic/blob/master/examples/src/mock/mock.rs
pub async fn spawn_and_connect_grpc() -> InferencerClient<Channel> {
    // these become components of our channel
    let (client, server) = tokio::io::duplex(1024);

    let inferencer = MockGrpc;
    tokio::spawn(async move {
        Server::builder()
            .add_service(InferencerServer::new(inferencer))
            .serve_with_incoming(tokio_stream::once(Ok::<_, std::io::Error>(server)))
            .await
    });

    let mut client = Some(client);

    let channel = Endpoint::try_from("http://[::1]:50052")
        .unwrap()
        .connect_with_connector(service_fn(move |_: Uri| {
            let client = client.take();

            async move {
                if let Some(client) = client {
                    Ok(TokioIo::new(client))
                } else {
                    Err(std::io::Error::other("client taken"))
                }
            }
        }))
        .await
        .unwrap();

    InferencerClient::new(channel)
}
