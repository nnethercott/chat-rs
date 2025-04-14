use grpc_service::{inferencer_server::InferencerServer, server::ModelServer, ModelSpec, ModelType};
use uuid::Uuid;
use std::net::SocketAddr;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let addr: SocketAddr = "[::1]:50051".parse()?;
    let mut ml_service = ModelServer::new();
    let registry: Vec<ModelSpec> = (0..32)
        .map(|_| ModelSpec {
            model_id: Uuid::new_v4().to_string(),
            model_type: ModelType::Image.into(),
        })
        .collect();
    ml_service.registry = registry;

    let server = Server::builder()
        .add_service(InferencerServer::new(ml_service))
        .serve(addr)
        .await?;

    Ok(())
}
