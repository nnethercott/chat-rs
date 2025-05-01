use grpc_service::{
    InferenceRequest, InferenceResponse, ModelSpec,
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
    type ListModelsStream = ReceiverStream<Result<ModelSpec, Status>>;

    async fn list_models(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::ListModelsStream>, Status> {
        let (tx, rx) = mpsc::channel(4);

        let model_list = vec![ModelSpec {
            model_id: "model".into(),
            model_type: 0,
        }];

        tokio::spawn(async move {
            for spec in model_list {
                tx.send(Ok(spec)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn add_models(
        &self,
        _request: tonic::Request<Streaming<ModelSpec>>,
    ) -> Result<Response<u64>, Status> {
        Ok(Response::new(42))
    }

    async fn run_inference(
        &self,
        _request: Request<InferenceRequest>,
    ) -> Result<Response<InferenceResponse>, Status> {
        let resp = InferenceResponse {
            logits: vec![1.0, 2.0, 3.0],
            timestamp: 42,
        };
        Ok(Response::new(resp))
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
