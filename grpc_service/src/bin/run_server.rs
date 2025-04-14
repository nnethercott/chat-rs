use grpc_service::{
    ModelSpec, ModelType, inferencer_server::InferencerServer, server::ModelServer,
};
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use uuid::Uuid;

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

    let (health_reporter, health_service) = health_reporter();
    health_reporter.set_serving::<InferencerServer<ModelServer>>().await;

    let server = Server::builder()
        .add_service(health_service)
        .add_service(InferencerServer::new(ml_service))
        .serve(addr)
        .await?;

    Ok(())
}
