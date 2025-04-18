use grpc_service::{
    ModelSpec, inferencer_client::InferencerClient, inferencer_server::InferencerServer,
    server::ModelServer, server::generate_random_registry,
};
use std::{net::SocketAddr, time::Duration};
use tokio::time;
use tokio_stream::StreamExt;
use tonic::{
    Request,
    transport::{Channel, Server},
};

// TODO: random port and store it here, then connect client in tests
pub struct TestServer {
    client: InferencerClient<Channel>,
}

impl TestServer {
    pub async fn new(addr: String) -> Self {
        // BAD ? sleep to allow server spawn
        time::sleep(Duration::from_millis(10)).await;
        let client = InferencerClient::connect(format!("http://{}", addr))
            .await
            .unwrap();
        Self { client }
    }

    pub async fn get_registry_models(&mut self) -> Vec<ModelSpec> {
        let stream = self
            .client
            .list_models(Request::new(()))
            .await
            .expect("failed to get stream")
            .into_inner();

        stream
            .filter_map(|i| i.ok())
            .collect::<Vec<ModelSpec>>()
            .await
    }
}

pub async fn spawn_server() -> TestServer {
    // TODO: randomize port & read from config
    let addr = "[::1]:50051";
    let socket_addr: SocketAddr = addr.parse().unwrap();

    // FIXME: get client
    let model_server = ModelServer::new(pg_client);

    // model server with fake registry
    tokio::spawn(async move {
        Server::builder()
            .add_service(InferencerServer::new(model_server))
            .serve(socket_addr)
            .await
            .unwrap();
    });

    TestServer::new(addr.to_owned()).await
}

// async fn ensure_server_ready(client: &InferencerClient){
//     todo!()
// }
