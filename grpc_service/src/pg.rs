use crate::ModelSpec;
use postgres_types::{FromSql, ToSql};

// a wrapper type for serializing composite type from db
#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "modelspec")]
pub struct PgModelSpec {
    model_id: String,
    model_type: i32,
}

impl Into<ModelSpec> for PgModelSpec {
    fn into(self) -> ModelSpec {
        ModelSpec {
            model_id: self.model_id,
            model_type: self.model_type,
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
