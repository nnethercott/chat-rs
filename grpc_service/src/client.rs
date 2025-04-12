use inference_service::{Null};
use inference_service::inferencer_client::InferencerClient;
use tokio_stream::StreamExt;
use tonic::Request;
use anyhow::Context;

pub mod inference_service {
    tonic::include_proto!("inferenceservice");
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut client = InferencerClient::connect("http://[::]:50051")
        .await
        .context("failed to connect to running server")?;

    let request = Request::new(Null{});
    let mut rx = client.list_models(request).await?.into_inner();
    while let Some(Ok(model)) = rx.next().await{
        println!("{:?}", model);
    }

    Ok(())
}
