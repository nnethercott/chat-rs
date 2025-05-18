use super::download_weights;
use crate::{hf::HfApiManager, modelpool::Opts, tokenizer::TokenOutputStream};
use anyhow::{Error as E, Result};
use candle_core::{DType, Device, Tensor};
use candle_transformers::{
    self,
    generation::LogitsProcessor,
    models::{mimi::candle_nn::VarBuilder, qwen2},
};
use tokenizers::{Token, Tokenizer};
use tracing::{debug, warn};

pub struct Model {
    pub device: Device,
    pub inner: qwen2::ModelForCausalLM, // make this an impl CausalLM bound
    pub tokenizer: TokenOutputStream,
}

impl Model {
    pub fn from_pretrained(model_id: String) -> Result<Self> {
        let device = Device::Cpu;
        let api = HfApiManager::new(model_id)?;

        // config
        let cfg_raw = api.get("config.json")?;
        let cfg: qwen2::Config = serde_json::from_slice(&std::fs::read(&cfg_raw)?)?;

        // model weights
        let weights = download_weights(&api)?;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&weights, DType::F32, &device)? };
        let model = qwen2::ModelForCausalLM::new(&cfg, vb)?;

        //tokenizer
        let tokenizer_raw = api.get("tokenizer.json")?;
        let tokenizer = Tokenizer::from_file(&tokenizer_raw).map_err(E::msg)?;

        Ok(Self {
            device,
            inner: model,
            tokenizer: TokenOutputStream::new(tokenizer),
        })
    }
}

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
        .filter_map(|&token| {
            match model.tokenizer.get_vocab(true).get(token).copied() {
                Some(t) => Some(t),
                None => {
                    warn!("cannot find the '{}' token", token);
                    None
                }
            }
        })
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

// #[async_trait]
// impl GenerativeModel for Arc<Mutex<Model>> {
//     async fn generate_stream(
//         &mut self,
//         prompt: String,
//         tx: tokio::sync::mpsc::Sender<String>,
//     ) -> Result<()> {
//         let model = Arc::clone(&self);
//
//         // CPU intensive task
//         tokio::task::spawn_blocking(move || {
//             let mut lock = model.lock().unwrap();
//             generate(&mut lock, prompt, 32, tx).unwrap();
//         })
//         .await;
//         Ok(())
//     }
// }
