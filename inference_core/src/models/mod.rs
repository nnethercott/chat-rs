use anyhow::Result;
use hf_hub::api::sync::ApiRepo;
use serde_json::Value;
use std::{collections::HashSet, path::PathBuf};

// models
pub mod qwen;

pub(crate) fn download_weights(repo: &ApiRepo) -> Result<Vec<PathBuf>> {
    let weights: Vec<PathBuf> = match repo.get("model.safetensors.index.json") {
        Ok(pathbuf) => {
            let json: Value = serde_json::from_slice(&std::fs::read(&pathbuf)?)?;

            if let Some(serde_json::Value::Object(map)) = json.get("weight_map") {
                let mut filenames = HashSet::new();
                //parse files from weight map
                for v in map.values() {
                    let f = v.as_str().unwrap();
                    filenames.insert(f.to_string());
                }
                // download weights
                let mut files = vec![];
                for f in filenames {
                    files.push(repo.get(&f)?);
                }

                files
            } else {
                anyhow::bail!("invalid model weight index");
            }
        }
        _ => vec![repo.get("model.safetensors")?],
    };

    Ok(weights)
}
//
