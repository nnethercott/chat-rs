use std::{panic, sync::{Arc, Mutex}};

use super::download_weights;
use crate::hf::{GenerativeModel, HfApiManager};
use anyhow::{Error as E, Result};
use async_trait::async_trait;
use candle_core::{DType, Device, Tensor};
use candle_transformers::{
    self,
    generation::LogitsProcessor,
    models::{mimi::candle_nn::VarBuilder, qwen2},
};
use tokenizers::Tokenizer;

pub struct Model {
    pub device: Device,
    pub inner: qwen2::ModelForCausalLM,
    pub tokenizer: Tokenizer,
    pub logits_processor: LogitsProcessor,
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
            tokenizer,
            logits_processor: LogitsProcessor::new(42, Some(0.8), None),
        })
    }
}

pub fn generate(
    model: &mut Model,
    prompt: String,
    max_new_tokens: usize,
    tx: tokio::sync::mpsc::Sender<u32>,
) -> Result<String> {
    let encoding = model.tokenizer.encode(prompt, true).map_err(E::msg)?;
    let mut tokens = encoding.get_ids().to_vec();
    println!("{:?}", tokens);

    // eos tokens for qwen2
    let eos_token = match model
        .tokenizer
        .get_vocab(true)
        .get("<|endoftext|>")
        .copied()
    {
        Some(token) => token,
        None => anyhow::bail!("cannot find the <|endoftext|> token"),
    };
    let eos_token2 = match model
        .tokenizer
        .get_vocab(true)
        .get("<|im_end|>")
        .copied()
    {
        Some(token) => token,
        None => anyhow::bail!("cannot find the <|im_end|> token"),
    };

    let mut generated_tokens = 0;

    for index in 0..max_new_tokens {
        println!("{index}");
        let context_size = if index > 0 { 1 } else { tokens.len() };
        let start_pos = tokens.len().saturating_sub(context_size);
        let ctxt = &tokens[start_pos..];
        let input_ids = Tensor::new(ctxt, &model.device)?.unsqueeze(0)?;

        // no attention mask (single seq at a time)
        let logits = model.inner.forward(&input_ids, start_pos)?;
        let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;

        // TODO: add a config for generation
        let logits = {
            let start_at = tokens.len().saturating_sub(64);
            candle_transformers::utils::apply_repeat_penalty(
                &logits,
                1.5, // https://huggingface.co/Qwen/Qwen3-4B `presence_penalty`
                &tokens[start_at..],
            )?
        };

        let next_token = model.logits_processor.sample(&logits)?;
        tokens.push(next_token);
        generated_tokens += 1;

        // apply callback
        // if tx.send(next_token).await.is_err() {
        //     panic!("failed to send token sync");
        // };

        if next_token == eos_token || next_token == eos_token2 {
            break;
        }
    }
    Ok("nate".into())
}

#[async_trait]
impl GenerativeModel for Arc<Mutex<Model>> {
    async fn generate_stream(
        &mut self,
        prompt: String,
        tx: tokio::sync::mpsc::Sender<u32>,
    ) -> Result<()> {
        let model = Arc::clone(&self);

        // CPU intensive task
        tokio::task::spawn_blocking(move || {
            let mut lock = model.lock().unwrap();
            generate(&mut lock, prompt, 16, tx).unwrap();
        })
        .await;
        Ok(())
    }
}
