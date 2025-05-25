pub mod config;
mod error;
pub mod proto;
pub mod server;

pub use error::Error;

tonic::include_proto!("inferenceservice");
const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("modelserver");
