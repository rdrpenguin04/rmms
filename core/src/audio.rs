#[cfg(feature = "cpal")]
use cpal::{traits::HostTrait, StreamError};

pub struct Engine;

#[cfg(feature = "cpal")]
#[derive(Debug)]
pub enum CreationError {
    NoOutputDevice,
}

#[allow(clippy::missing_errors_doc)] // TODO: Document
#[allow(clippy::missing_panics_doc)] // TODO: Don't panic
#[cfg(feature = "cpal")]
pub fn spawn_engine(_stream_err: impl FnMut(StreamError)) -> Result<(), CreationError> {
    let host = cpal::default_host();
    let _device = host
        .default_output_device()
        .ok_or(CreationError::NoOutputDevice)?;
    todo!()
}
