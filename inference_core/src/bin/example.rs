use anyhow::Error as E;
use anyhow::Result;
use candle_core::DType;
use candle_core::Tensor;
use candle_transformers::generation::LogitsProcessor;
use inference_core::models::qwen::Model;

fn main() {
    println!(
        "avx: {}, neon: {}, simd128: {}, f16c: {}",
        candle_core::utils::with_avx(),
        candle_core::utils::with_neon(),
        candle_core::utils::with_simd128(),
        candle_core::utils::with_f16c()
    );

    let rt = tokio::runtime::Runtime::new().unwrap();
    let model = rt.block_on(async {
        Model::from_pretrained("Qwen/Qwen2-0.5B".into(), candle_core::Device::Cpu).unwrap()
    });

    // runs outside of tokios executor
    run(model, "tell me a joke", 32).expect("works");
}

fn run(mut model: Model, prompt: &str, sample_len: usize) -> Result<()> {
    let mut tokens = model
        .tokenizer
        .encode(prompt, true)
        .map_err(E::msg)?
        .get_ids()
        .to_vec();

    let eos_token = match model
        .tokenizer
        .get_vocab(true)
        .get("<|endoftext|>")
        .copied()
    {
        Some(token) => token,
        None => anyhow::bail!("cannot find the <|endoftext|> token"),
    };
    let eos_token2 = eos_token;

    for index in 0..sample_len {
        println!("{index}");
        let context_size = if index > 0 { 1 } else { tokens.len() };
        let start_pos = tokens.len().saturating_sub(context_size);
        let ctxt = &tokens[start_pos..];
        let input = Tensor::new(ctxt, &model.device)?.unsqueeze(0)?;
        let logits = model.inner.forward(&input, start_pos)?;
        let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
        let logits = {
            let start_at = tokens.len().saturating_sub(64);
            candle_transformers::utils::apply_repeat_penalty(&logits, 1.5, &tokens[start_at..])?
        };

        let mut logits_processor = LogitsProcessor::new(42, Some(0.4), None);
        let next_token = logits_processor.sample(&logits)?;
        tokens.push(next_token);
        if next_token == eos_token || next_token == eos_token2 {
            break;
        }
    }
    Ok(())
}
