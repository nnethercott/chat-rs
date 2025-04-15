use grpc_service::{
    configuration::generate_random_registry, inferencer_client::InferencerClient, inferencer_server::InferencerServer, server::ModelServer, ModelSpec, ModelType
};
use tokio::time;
use std::{net::SocketAddr, time::Duration};
use tokio_stream::StreamExt;
use tonic::{
    Request,
    transport::{Channel, Server},
};
use uuid::Uuid;

// TODO: random port and store it here, then connect client in tests
pub struct TestServer {
    client: InferencerClient<Channel>,
}

impl TestServer {
    pub async fn new(addr: String) -> Self
    {
        // BAD ?
        time::sleep(Duration::from_millis(10)).await;
        let client = InferencerClient::connect(format!("http://{}", addr)).await.unwrap();
        Self { client }
    }

    pub async fn get_registry_models(&mut self) -> Vec<ModelSpec> {
        let stream = self
            .client
            .list_models(Request::new(()))
            .await
            .expect("failed to get stream")
            .into_inner();

        let mut stream = stream.filter_map(|i| i.ok());

        let mut models = vec![];
        while let Some(val) = stream.next().await{
            models.push(val);
        }
        models
    }
}

pub async fn spawn_server() -> TestServer {
    // TODO: randomize port & read from config
    let addr = "[::1]:50051";
    let socket_addr: SocketAddr = addr.parse().unwrap();

    // model server with fake registry
    let mut ml_service = ModelServer::new();
    ml_service.registry = generate_random_registry();

    tokio::spawn(async move {
        let server = ModelServer::new();
        Server::builder()
            .add_service(InferencerServer::new(ml_service))
            .serve(socket_addr)
            .await
            .unwrap();
    });

    TestServer::new(addr.to_owned()).await
}

// async fn ensure_server_ready(client: &InferencerClient){
//     todo!()
// }
