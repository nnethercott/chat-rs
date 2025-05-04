use anyhow::Result;
use async_trait::async_trait;
use candle_core::{DType, Device};
use candle_transformers::models::{mimi::candle_nn::VarBuilder, qwen2};
use tokenizers::Tokenizer;

use crate::hf::{GenerativeModel, HfApiManager};
use super::download_weights;

pub struct Model {
    model: qwen2::Model,
    tokenizer: Tokenizer,
}

impl Model {
    pub async fn from_pretrained(model_id: String, device: &Device) -> Result<Self> {
        let api = HfApiManager::new(model_id)?;
        // config
        let cfg_raw = api.get("config.json").await?;
        let cfg: qwen2::Config = serde_json::from_slice(&std::fs::read(&cfg_raw)?)?;
        // model weights
        let weights = download_weights(&api).await?;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&weights, DType::F32, device)? };
        let model = qwen2::Model::new(&cfg, vb)?;
        //tokenizer
        let tokenizer_raw = api.get("tokenizer.json").await?;
        let tokenizer = Tokenizer::from_file(&tokenizer_raw).map_err(anyhow::Error::msg)?;

        Ok(Self { model, tokenizer })
    }
}

#[async_trait]
impl GenerativeModel for Model {
    async fn generate_stream(
        &self,
        input_ids: crate::hf::Tokens,
        tx: tokio::sync::mpsc::Sender<u32>,
    ) {
        todo!()
    }
}
