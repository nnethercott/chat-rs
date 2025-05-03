use crate::ModelSpec;
use sqlx::{FromRow, Row, postgres::PgRow};

// serialization to postgres
impl FromRow<'_, PgRow> for ModelSpec {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            model_id: row.try_get("model_id")?,
            model_type: row.try_get("model_type")?,
        })
    }
}
