use std::{
    fs::File,
    io::{BufReader, Write},
};

use bitvec::{prelude::Msb0, vec::BitVec};
use clap::Parser;
use flacenc::component::BitRepr;
use rmms_core::{
    audio,
    mmp::{Mmp, MmpReadError},
};

/// Standalone Exporter for RMMS projects
#[derive(Parser, Debug)]
struct Args {
    filename: String,
}

struct RenderOutput {
    data: Box<[i32]>,
    index: usize,
}

impl flacenc::source::Source for RenderOutput {
    fn channels(&self) -> usize {
        1
    }

    fn bits_per_sample(&self) -> usize {
        32
    }

    fn sample_rate(&self) -> usize {
        44100
    }

    fn read_samples(
        &mut self,
        dest: &mut flacenc::source::FrameBuf,
    ) -> Result<usize, flacenc::error::SourceError> {
        let mut count = 0;
        for x in dest.channel_slice_mut(0) {
            if self.index >= self.data.len() {
                break;
            }
            *x = self.data[self.index];
            self.index += 1;
            count += 1;
        }
        Ok(count)
    }
}

impl RenderOutput {
    fn new(data: Box<[i32]>) -> Self {
        Self { data, index: 0 }
    }
}

fn main() -> Result<(), MmpReadError> {
    let args = Args::parse();

    // ! TEMPORARY TEST CODE ! //
    let mut audio_engine = audio::Engine::new();
    let mut output = [0; 44100];
    audio_engine.play = true;
    audio_engine.fill_buffer(&mut output);
    let render_output = RenderOutput::new(Box::from(output));
    let encoder = flacenc::config::Encoder::default();
    let stream =
        flacenc::coding::encode_with_fixed_block_size(&encoder, render_output, 2048).unwrap();
    let mut bitvec: BitVec<u8, Msb0> = BitVec::with_capacity(stream.count_bits());
    stream.write(&mut bitvec).unwrap();
    File::create("output.flac")
        .unwrap()
        .write_all(bitvec.as_raw_slice())
        .unwrap();
    // ! END TEST CODE ! //

    let file = Mmp::load(BufReader::new(File::open(args.filename)?))?;
    println!("{file:?}");
    Ok(())
}
