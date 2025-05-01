use crate::ModelSpec;
use postgres_types::{FromSql, ToSql};

// a wrapper type for serializing composite type from db
#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "modelspec")]
pub struct PgModelSpec {
    pub model_id: String,
    pub model_type: i32,
}

impl From<PgModelSpec> for ModelSpec {
    fn from(value: PgModelSpec) -> Self {
        ModelSpec {
            model_id: value.model_id,
            model_type: value.model_type,
        }
    }
}

impl From<ModelSpec> for PgModelSpec {
    fn from(value: ModelSpec) -> Self {
        Self {
            model_id: value.model_id,
            model_type: value.model_type,
        }
    }
}
