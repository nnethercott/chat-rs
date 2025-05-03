use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // needed for reflection
    let descriptor_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("modelserver.bin");

    tonic_build::configure()
        .build_server(true)
        // add serde for axum endpoints
        .type_attribute(
            "ModelSpec",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .file_descriptor_set_path(descriptor_path)
        .compile_protos(&["proto/inference_service.proto"], &["proto/"])?;

    Ok(())
}
