pub mod config;
pub mod server;
pub mod telemetry;
mod error;

pub use error::Error;

tonic::include_proto!("inferenceservice");
const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("modelserver");
