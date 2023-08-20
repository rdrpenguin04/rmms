use serde::{de, Deserialize};
use std::borrow::Cow;

use super::WaveType;

// TODO: impl Default to be the LMMS default settings
#[derive(Debug, Default)]
pub struct Organic {
    pub vol: f32,
    pub foldback: f32,
    pub oscillators: [Oscillator; 8],
}

pub struct OrganicVisitor;

impl<'de> de::Visitor<'de> for OrganicVisitor {
    type Value = Organic;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a organic element")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut result = Organic::default();

        while let Some(key) = map.next_key::<Cow<str>>()? {
            if key == "key" {
                continue;
            } else if key == "@vol" {
                result.vol = map.next_value::<Cow<str>>()?.parse().unwrap();
            } else if key == "@foldback" {
                result.foldback = map.next_value::<Cow<str>>()?.parse().unwrap();
            } else if key == "@num_osc" {
                continue;
            } else {
                let (name, idx) = key.split_at(key.len() - 1);
                let idx: usize = idx
                    .parse()
                    .expect("expected a organic instrument parameter"); // TODO: proper error handling
                let osc = &mut result.oscillators[idx];

                match name {
                    "@vol" => osc.vol = map.next_value::<Cow<str>>()?.parse().unwrap(),
                    "@pan" => osc.pan = map.next_value::<Cow<str>>()?.parse().unwrap(),
                    "@newharmonic" => osc.harmonic = map.next_value::<Cow<str>>()?.parse().unwrap(),
                    "@newdetune" => osc.detune = map.next_value::<Cow<str>>()?.parse().unwrap(),
                    "@wavetype" => {
                        osc.wave_type = match &*map.next_value::<String>()? {
                            "0" => WaveType::SineWave,
                            "1" => WaveType::TriangleWave,
                            "2" => WaveType::SawWave,
                            "3" => WaveType::SquareWave,
                            "4" => WaveType::MoogSawWave,
                            "5" => WaveType::ExponentialWave,
                            "6" => WaveType::WhiteNoise,
                            _ => panic!("invalid wavetype"),
                        }
                    }
                    x => panic!("expected a organic instrument parameter, got {x}"), // TODO: proper error handling
                }
            }
        }

        Ok(result)
    }
}

impl<'de> Deserialize<'de> for Organic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(OrganicVisitor)
    }
}

#[derive(Debug, Default)]
pub struct Oscillator {
    pub vol: f32,
    pub pan: f32,
    pub harmonic: i32,
    pub detune: f32,
    pub wave_type: WaveType,
}
