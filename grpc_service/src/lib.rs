pub mod config;
mod error;
pub mod server;
pub mod pg;

pub use error::Error;

tonic::include_proto!("inferenceservice");
const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("modelserver");
