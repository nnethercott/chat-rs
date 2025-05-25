use crate::SamplingOpts;
use inference_core::modelpool::Opts;

impl From<SamplingOpts> for Opts {
    fn from(val: SamplingOpts) -> Self {
        Opts {
            max_new_tokens: val.max_new_tokens,
            temperature: val.temperature,
            eos_tokens: val.eos_tokens,
            top_k: val.top_k,
            top_p: val.top_p,
            repeat_penalty: val.repeat_penalty,
        }
    }
}

// serialization to postgres
// impl FromRow<'_, PgRow> for ModelSpec {
//     fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
//         Ok(row.try_get("spec")?)
//     }
// }
