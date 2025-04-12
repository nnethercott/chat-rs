fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/inference_service.proto")?;
    Ok(())
}
