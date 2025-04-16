use grpc_service::{config::Settings, inferencer_client::InferencerClient, server::run_server};
use std::time::Duration;
use tokio_stream::StreamExt;
use tonic::Request;

// FIXME: find a way to link the addr better ...

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let server_handle = tokio::spawn(async {
        let config = Settings::new();
        if run_server(config).await.is_err() {
            //pass
        }
    });

    let client_handle = tokio::spawn(async {
        // let the server spawn ...
        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut client = InferencerClient::connect(format!("http://{}", "[::1]:50051"))
            .await
            .unwrap();

        let request = Request::new(());
        let mut rx = client.list_models(request).await.unwrap().into_inner();
        while let Some(Ok(model)) = rx.next().await {
            println!("{:?}", model);
        }
    });

    tokio::select! {
        _ = server_handle => {},
        _ = client_handle => {} // will complete first
    }

    Ok(())
}
