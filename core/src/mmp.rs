pub mod instrument;

use std::io::{self, BufRead};

use quick_xml::impl_deserialize_for_internally_tagged_enum;
use serde::Deserialize;
use thiserror::Error;

use self::instrument::Instrument;

#[derive(Debug, Error)]
pub enum MmpReadError {
    #[error("error in deserialization: {0}")]
    DeError(#[from] quick_xml::DeError),
    #[error("error in reading: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Debug, Deserialize)]
pub struct Mmp {
    pub head: Head,
    pub song: Song,
}

impl Mmp {
    pub fn load(file: impl BufRead) -> Result<Self, MmpReadError> {
        quick_xml::de::from_reader(file).map_err(MmpReadError::from)
    }
}

#[derive(Debug, Deserialize)]
pub struct Head {
    #[serde(rename = "@bpm")]
    pub tempo: f32,
    #[serde(rename = "@masterpitch")]
    pub master_pitch: f32,
    #[serde(rename = "@mastervol")]
    pub master_vol: f32,
    #[serde(rename = "@timesig_denominator")]
    pub timesig_denominator: i32,
    #[serde(rename = "@timesig_numerator")]
    pub timesig_numerator: i32,
}

#[derive(Debug, Deserialize)]
pub struct Song {
    #[serde(rename = "trackcontainer")]
    pub track_container: TrackContainer,
    #[serde(rename = "projectnotes")]
    pub project_notes: String,
}

#[derive(Debug, Deserialize)]
pub struct TrackContainer {
    #[serde(default, rename = "track")]
    pub tracks: Vec<Track>,
}

#[derive(Debug)]
pub enum Track {
    InstrumentTrack(TrackInstrumentTrack),
    BBTrack(TrackBBTrack),
    SampleTrack(TrackSampleTrack),
    AutomationTrack(TrackAutomationTrack),
}

impl_deserialize_for_internally_tagged_enum! {
    Track, "@type",
    ("0" => InstrumentTrack(TrackInstrumentTrack)),
    ("1" => BBTrack(TrackBBTrack)),
    ("2" => SampleTrack(TrackSampleTrack)),
    ("5" => AutomationTrack(TrackAutomationTrack)),
}

#[derive(Debug, Deserialize)]
pub struct TrackInstrumentTrack {
    #[serde(rename = "@solo")]
    pub solo: bool,
    #[serde(rename = "@muted")]
    pub muted: bool,
    #[serde(rename = "@mutedBeforeSolo")]
    pub muted_before_solo: bool,
    #[serde(rename = "instrumenttrack")]
    pub instrument_track: InstrumentTrack,
    #[serde(rename = "midiclip")]
    #[serde(default)]
    pub clips: Vec<MidiClip>,
}

#[derive(Debug, Deserialize)]
pub struct TrackBBTrack {}

#[derive(Debug, Deserialize)]
pub struct TrackSampleTrack {}

#[derive(Debug, Deserialize)]
pub struct TrackAutomationTrack {}

#[derive(Debug, Deserialize)]
pub struct InstrumentTrack {
    #[serde(rename = "@vol")]
    pub vol: f32,
    #[serde(rename = "@pan")]
    pub pan: f32,
    #[serde(rename = "@firstkey")]
    pub first_key: i32,
    #[serde(rename = "@lastkey")]
    pub last_key: i32,
    #[serde(rename = "@basenote")]
    pub base_note: i32,
    pub instrument: Instrument,
}

#[derive(Debug, Deserialize)]
pub struct MidiClip {
    #[serde(rename = "@pos")]
    pub pos: u32,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@muted")]
    pub muted: bool,
    #[serde(default)]
    #[serde(rename = "note")]
    pub notes: Vec<Note>,
}

#[derive(Debug, Deserialize)]
pub struct Note {
    #[serde(rename = "@pos")]
    pub pos: u32,
    #[serde(rename = "@len")]
    pub len: u32,
    #[serde(rename = "@key")]
    pub key: u32,
    #[serde(rename = "@vol")]
    pub vol: f32,
    #[serde(rename = "@pan")]
    pub pan: f32,
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::BufReader};

    use super::Mmp;

    #[test]
    fn format() {
        dbg!(Mmp::load(BufReader::new(File::open("../test/format.mmp").unwrap())).unwrap());
    }
}
