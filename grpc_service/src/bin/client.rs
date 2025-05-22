use std::io::Write;

use grpc_service::inferencer_client::InferencerClient;
use tokio_stream::StreamExt;
use tonic::Request;

#[tokio::main]
async fn main() {
    let client_handle = tokio::spawn(async {
        //NOTE: run this after server is spawned

        let mut client = InferencerClient::connect(format!("http://{}", "[::1]:50051"))
            .await
            .unwrap();

        let request = Request::new(());

        // add some models to the server
        let n = client
            .add_models(tokio_stream::iter(vec![
                "alibaba/qwen2.5".into(),
                "jina/embeddingsv2".into(),
                "meta/llama4".into(),
            ]))
            .await
            .expect("insert failed")
            .into_inner();

        println!("{n} models successfully added");

        let mut rx = client.list_models(request).await.unwrap().into_inner();
        while let Some(Ok(model)) = rx.next().await {
            println!("{:?}", model);
        }

        let mut rx = client
            .generate_streaming(Request::new("tell me a funny joke".into()))
            .await
            .unwrap()
            .into_inner();

        while let Some(Ok(word)) = rx.next().await {
            print!("{word}");
            std::io::stdout().flush().unwrap();
        }
        println!("\n");
    });

    tokio::select! {
        _ = client_handle => {},
    }
}
