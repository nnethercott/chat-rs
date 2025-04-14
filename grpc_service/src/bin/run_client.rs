use anyhow::Context;
use tokio_stream::StreamExt;
use tonic::Request;
use grpc_service::inferencer_client::InferencerClient;

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut client = InferencerClient::connect("http://[::]:50051")
        .await
        .context("failed to connect to running server")?;

    let request = Request::new(());
    let mut rx = client.list_models(request).await?.into_inner();
    while let Some(Ok(model)) = rx.next().await {
        println!("{:?}", model);
    }

    Ok(())
}
