use crate::helpers::spawn_server;
use grpc_service::inferencer_client::InferencerClient;
use uuid::Uuid;

// TODO: init tracing in tests!

// NOTE: this is not reaallly a test
#[tokio::test]
async fn test_add_and_list_models_works() {
    let mut ts = spawn_server().await;

    let mut models = vec![Uuid::new_v4().to_string()];

    // add some models
    let m = ts.add_models_to_registry(models).await;

    // retrieve those models
    models = ts.get_registry_models().await;

    assert_eq!(m as usize, 1);
    assert_eq!(models.len() as usize, 1);
}

#[tokio::test]
async fn multiple_clients_single_server() {
    let ts = spawn_server().await;

    let a1 = ts.addr.clone();
    let a2 = ts.addr.clone();

    // manually connect with two clients
    let h1 = tokio::spawn(async move {
        let _client = InferencerClient::connect(format!("http://{}", &a1))
            .await
            .unwrap();
    });
    let h2 = tokio::spawn(async move {
        let _client = InferencerClient::connect(format!("http://{}", &a2))
            .await
            .unwrap();
    });

    let _ = tokio::join!(h1, h2);
}
