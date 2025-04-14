use grpc_service::{server::ModelServer, ModelSpec, ModelType, inferencer_server::InferencerServer};
use std::net::SocketAddr;
use tonic::transport::Server;
use uuid::Uuid;

// TODO: random port and store it here, then connect client in tests
pub struct TestServer {
    // client?
}

pub async fn spawn_server() -> TestServer {
    // TODO: randomize port & read from config
    let addr: SocketAddr = "[::1]:50051".parse().unwrap();

    // model server with fake registry
    let mut ml_service = ModelServer::new();
    let registry: Vec<ModelSpec> = (0..32)
        .map(|_| ModelSpec {
            model_id: Uuid::new_v4().to_string(),
            model_type: ModelType::Image.into(),
        })
        .collect();
    ml_service.registry = registry;

    tokio::spawn(async move {
        let server = ModelServer::new();
        Server::builder()
            .add_service(InferencerServer::new(ml_service))
            .serve(addr)
            .await
            .unwrap();
    });

    TestServer {}
}
