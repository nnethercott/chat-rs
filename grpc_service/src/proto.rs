use crate::SamplingOpts;
use inference_core::modelpool::Opts;

impl Into<Opts> for SamplingOpts {
    fn into(self) -> Opts {
        Opts {
            max_new_tokens: self.max_new_tokens,
            temperature: self.temperature,
            eos_tokens: self.eos_tokens,
            top_k: self.top_k,
            top_p: self.top_p,
            repeat_penalty: self.repeat_penalty,
        }
    }
}

// serialization to postgres
// impl FromRow<'_, PgRow> for ModelSpec {
//     fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
//         Ok(row.try_get("spec")?)
//     }
// }
