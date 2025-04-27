use grpc_service::{
    ModelSpec,
    config::get_config,
    inferencer_client::InferencerClient,
    inferencer_server::InferencerServer,
    server::{ModelServer, connect_to_db},
};
use std::{env, time::Duration};
use tokio::{net::TcpListener, time};
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
        // BAD ? sleep to allow server spawn (could loop)
        time::sleep(Duration::from_millis(10)).await;
        let client = InferencerClient::connect(format!("http://{}", addr))
            .await
            .unwrap();
        Self { client }
    }

    pub async fn add_models_to_registry(&mut self, models: Vec<ModelSpec>) -> u64 {
        self.client
            .add_models(tokio_stream::iter(models))
            .await
            .unwrap()
            .into_inner()
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

fn set_config_env_var() {
    // wokrdir in tests different than main crate
    let config_path = env::current_dir()
        .unwrap()
        .join(format!("config/local.yaml"));

    unsafe {
        env::set_var("CONFIG_FILE", &config_path);
    }
}

pub async fn spawn_server() -> TestServer {
    set_config_env_var();

    //TODO: modify config to create new table for tests
    let config = get_config().unwrap();
    let pgpool = connect_to_db(&config.db).await.unwrap();
    let model_server = ModelServer::new(pgpool);

    // set listener on random free port
    let listener = TcpListener::bind("[::1]:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // model server with fake registry
    tokio::spawn(async move {
        Server::builder()
            .add_service(InferencerServer::new(model_server))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    TestServer::new(addr.to_string()).await
}
