pub mod config;
pub mod server;
pub mod telemetry;

tonic::include_proto!("inferenceservice");
const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("modelserver");
