use crate::{ModelSpec, ModelType};
use uuid::Uuid;

pub struct Settings;
pub struct DatabaseSettings;

pub fn generate_random_registry() -> Vec<ModelSpec> {
    (0..32)
        .map(|_| ModelSpec {
            model_id: Uuid::new_v4().to_string(),
            model_type: ModelType::Image.into(),
        })
        .collect()
}
