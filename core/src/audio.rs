#[cfg(feature = "cpal")]
use cpal::{StreamError, traits::HostTrait};

pub struct Engine;

#[cfg(feature = "cpal")]
#[derive(Debug)]
pub enum CreationError {
    NoOutputDevice,
}

#[cfg(feature = "cpal")]
pub fn spawn_engine(_stream_err: impl FnMut(StreamError)) -> Result<(), CreationError> {
    let host = cpal::default_host();
    let _device = host.default_output_device().ok_or(CreationError::NoOutputDevice)?;
    todo!()
}
