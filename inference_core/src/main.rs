
use candle_core::{DType, Device};
use candle_transformers::models::{
    mimi::candle_nn::VarBuilder,
    qwen2::{Config as Qwen2Config, Model as Qwen2},
};
use inference_core::hf::HfApiManager;

const MODEL_ID: &str = "Qwen/Qwen2.5-0.5B-Instruct";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let device = Device::Cpu;
    let api = HfApiManager::new(MODEL_ID.to_string())?;

    let mut pathbuf = api.download("config.json").await?;
    let cfg: Qwen2Config = serde_json::from_slice(&std::fs::read(pathbuf)?)?;

    pathbuf = api.get("model.safetensors").await?;
    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[pathbuf], DType::F32, &device)? };
    let _model = Qwen2::new(&cfg, vb);

    Ok(())
}
