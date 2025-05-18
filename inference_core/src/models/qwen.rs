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
