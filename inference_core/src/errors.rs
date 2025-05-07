use crate::modelpool::SendBackMessage;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    HfError(#[from] anyhow::Error),

    #[error("model failed to load")]
    ModelLoadError(#[source] anyhow::Error),

    #[error(transparent)]
    TaskScheduleError(#[from] crossbeam_channel::SendError<SendBackMessage>),

    #[error("failed with reason: {reason}")]
    Other { reason: &'static str },
}
