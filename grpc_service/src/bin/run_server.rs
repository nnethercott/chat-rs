use grpc_service::{configuration::Settings, server::run_server};

#[tokio::main]
async fn main(){
    let handle = tokio::spawn(async {
        run_server(Settings).await
    });

    tokio::select! {
        _ = handle => {}
    }
}
