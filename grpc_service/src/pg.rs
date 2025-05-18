use crate::ModelSpec;
use sqlx::{FromRow, Row, postgres::PgRow};

// serialization to postgres
impl FromRow<'_, PgRow> for ModelSpec {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(row.try_get("spec")?)
    }
}
