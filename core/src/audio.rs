#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]

#[cfg(feature = "cpal")]
use cpal::{traits::HostTrait, StreamError};

pub trait Sample {
    fn from_sample(s: impl Sample) -> Self;
    fn to_f32(self) -> f32;
    fn to_i16(self) -> i16;
    fn to_i32(self) -> i32;
}

impl Sample for f32 {
    #[inline]
    fn from_sample(s: impl Sample) -> Self {
        s.to_f32()
    }

    #[inline]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline]
    fn to_i16(self) -> i16 {
        (self * Self::from(i16::MAX)).round() as i16
    }

    #[inline]
    fn to_i32(self) -> i32 {
        (self * i32::MAX as Self).round() as i32
    }
}

impl Sample for i16 {
    #[inline]
    fn from_sample(s: impl Sample) -> Self {
        s.to_i16()
    }

    #[inline]
    fn to_f32(self) -> f32 {
        f32::from(self) / f32::from(Self::MAX)
    }

    #[inline]
    fn to_i16(self) -> i16 {
        self
    }

    #[inline]
    fn to_i32(self) -> i32 {
        i32::from(self) * i32::from(Self::MAX)
    }
}

impl Sample for i32 {
    #[inline]
    fn from_sample(s: impl Sample) -> Self {
        s.to_i32()
    }

    #[inline]
    fn to_f32(self) -> f32 {
        self as f32 / Self::MAX as f32
    }

    #[inline]
    fn to_i16(self) -> i16 {
        (self / Self::from(i16::MAX)) as i16
    }

    #[inline]
    fn to_i32(self) -> i32 {
        self
    }
}

pub struct Engine {
    pub play: bool,
}

impl Engine {
    pub fn fill_buffer<T: Sample>(&mut self, buf: &mut [T]) {
        for sample in buf {
            *sample = Sample::from_sample(0);
        }
    }

    #[must_use]
    pub const fn new() -> Self {
        Self { play: false }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

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
