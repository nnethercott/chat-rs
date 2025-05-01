use crate::helpers::spawn_server;
use grpc_service::{ModelSpec, ModelType};
use uuid::Uuid;

// NOTE: this is not reaallly a test
#[tokio::test]
async fn test_add_and_list_models_works() {
    let mut ts = spawn_server().await;

    let mut models = vec![ModelSpec {
        model_id: Uuid::new_v4().to_string(),
        model_type: ModelType::Text.into(),
    }];
    let n = models.len();

    // add some models
    let m = ts.add_models_to_registry(models).await;

    // retrieve those models
    models = ts.get_registry_models().await;

    assert_eq!(m as usize, n);
    // assert_eq!(models.len() as usize, n);
}
