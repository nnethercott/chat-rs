use crate::{modelpool::Opts, models::qwen::Model};
use anyhow::{Error as E, Result};
use candle_core::{DType, Tensor};
use candle_transformers::generation::LogitsProcessor;
use tracing::{debug, warn};

pub fn generate(
    model: &mut Model, // TODO: replace this with a trait bound
    prompt: String,
    opts: Opts,
    tx: Option<tokio::sync::mpsc::Sender<String>>,
) -> Result<String> {
    let mut logits_processor = LogitsProcessor::new(42, opts.temperature, opts.top_p);
    let eos_tokens: Vec<u32> = opts
        .eos_tokens
        .iter()
        .filter_map(
            |token| match model.tokenizer.get_vocab(true).get(token).copied() {
                Some(t) => Some(t),
                None => {
                    warn!("cannot find the '{}' token", token);
                    None
                }
            },
        )
        .collect();

    let encoding = model.tokenizer.encode(prompt, true).map_err(E::msg)?;
    let mut tokens = encoding.get_ids().to_vec();

    for index in 0..opts.max_new_tokens {
        let context_size = if index > 0 { 1 } else { tokens.len() };
        let start_pos = tokens.len().saturating_sub(context_size);
        let ctxt = &tokens[start_pos..];
        let input_ids = Tensor::new(ctxt, &model.device)?.unsqueeze(0)?;

        // no attention mask (single seq at a time)
        let logits = model.inner.forward(&input_ids, start_pos)?;
        let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;

        let logits = {
            let start_at = tokens.len().saturating_sub(64);
            candle_transformers::utils::apply_repeat_penalty(
                &logits,
                opts.repeat_penalty.unwrap_or(1.0),
                &tokens[start_at..],
            )?
        };

        let next_token = logits_processor.sample(&logits)?;
        tokens.push(next_token);

        // maybe stream back
        if let Some(send_back) = tx.as_ref() {
            if let Some(word) = model.tokenizer.next_token(next_token)? {
                if let Err(e) = send_back.try_send(word) {
                    debug!(tokio_handle=?tokio::runtime::Handle::try_current());
                    panic!("failed to send token sync\n{:?}", e);
                };
            }
        }

        if eos_tokens.contains(&next_token) {
            break;
        }
    }
    model.tokenizer.decode(&tokens, false).map_err(E::msg)
}
