use std::{
    fs::File,
    io::{self, BufReader},
};

use clap::Parser;
use lmms2_core::format::ProjectFile;

/// Standalone Exporter for LMMS projects
#[derive(Parser, Debug)]
struct Args {
    filename: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let file = ProjectFile::load(BufReader::new(File::open(args.filename)?))?;
    println!("{:?}", file);
    Ok(())
}
