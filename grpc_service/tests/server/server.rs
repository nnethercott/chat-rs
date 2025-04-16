use crate::helpers::spawn_server;

// NOTE: this is not reaallly a test
#[tokio::test]
async fn list_model_endpoint_works() {
    let mut ts = spawn_server().await;

    let models = ts.get_registry_models().await;
    for model in &models[..] {
        println!("{:?}", model);
    }
}
