use core::fmt;

use serde::{de, Deserialize, Deserializer};
use std::borrow::Cow;

use crate::util::Stereo;

use super::WaveType;

// TODO: impl Default to be the LMMS default settings
#[derive(Debug, Default)]
pub struct TripleOscillator {
    pub oscillators: [Oscillator; 3],
}

impl<'de> Deserialize<'de> for TripleOscillator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TripleOscillatorVisitor)
    }
}

struct TripleOscillatorVisitor;

impl<'de> de::Visitor<'de> for TripleOscillatorVisitor {
    type Value = TripleOscillator;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a tripleoscillator element")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut result = TripleOscillator::default();

        while let Some(key) = map.next_key::<Cow<str>>()? {
            if key == "key" {
                continue;
            }
            let (name, idx) = key.split_at(key.len() - 1);
            let idx: usize = idx
                .parse()
                .expect("expected a tripleoscillator instrument parameter"); // TODO: proper error handling
            match name {
                "@vol" => {
                    result.oscillators[idx].vol = map.next_value::<String>()?.parse().unwrap()
                }
                "@pan" => {
                    result.oscillators[idx].pan = map.next_value::<String>()?.parse().unwrap()
                }
                "@coarse" => {
                    result.oscillators[idx].coarse = map.next_value::<String>()?.parse().unwrap()
                }
                "@finel" => {
                    result.oscillators[idx].fine.l = map.next_value::<String>()?.parse().unwrap()
                }
                "@finer" => {
                    result.oscillators[idx].fine.r = map.next_value::<String>()?.parse().unwrap()
                }
                "@phoffset" => {
                    result.oscillators[idx].ph_off = map.next_value::<String>()?.parse().unwrap()
                }
                "@stphdetun" => {
                    result.oscillators[idx].st_ph_detune =
                        map.next_value::<String>()?.parse().unwrap()
                }
                "@wavetype" => {
                    if matches!(result.oscillators[idx].wave_type, WaveType::Uninit) {
                        result.oscillators[idx].wave_type = match &*map.next_value::<String>()? {
                            "0" => WaveType::SineWave,
                            "1" => WaveType::TriangleWave,
                            "2" => WaveType::SawWave,
                            "3" => WaveType::SquareWave,
                            "4" => WaveType::MoogSawWave,
                            "5" => WaveType::ExponentialWave,
                            "6" => WaveType::WhiteNoise,
                            "7" => WaveType::UserDefinedWave(String::from("<uninit>")),
                            _ => panic!("invalid wavetype"),
                        }
                    }
                }
                "@userwavefile" => match &mut result.oscillators[idx].wave_type {
                    WaveType::UserDefinedWave(x) => {
                        *x = map.next_value()?;
                    }
                    x @ WaveType::Uninit => {
                        let path: String = map.next_value()?;
                        if !path.is_empty() {
                            *x = WaveType::UserDefinedWave(path);
                        }
                    }
                    _ => {}
                },
                "@modalgo" => {
                    result.oscillators[idx - 1].mod_algo = match &*map.next_value::<String>()? {
                        "0" => ModulationAlgo::PhaseModulation,
                        "1" => ModulationAlgo::AmplitudeModulation,
                        "2" => ModulationAlgo::SignalMix,
                        "3" => ModulationAlgo::SynchronizedBySubOsc,
                        "4" => ModulationAlgo::FrequencyModulation,
                        _ => panic!("invalid modalgo"),
                    }
                }
                "@useWaveTable" => {
                    result.oscillators[idx - 1].use_wave_table =
                        match &*map.next_value::<String>()? {
                            "0" => false,
                            "1" => true,
                            _ => panic!("expected 0 or 1"),
                        }
                }
                x => panic!("expected a tripleoscillator instrument parameter, got {x}"), // TODO: proper error handling
            }
        }

        Ok(result)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Oscillator {
    pub vol: f32,
    pub pan: f32,
    pub coarse: i32,
    pub fine: Stereo<f32>,
    pub ph_off: f32,
    pub st_ph_detune: f32,
    pub wave_type: WaveType,
    pub use_wave_table: bool,
    pub mod_algo: ModulationAlgo,
}

#[derive(Clone, Debug, Default)]
pub enum ModulationAlgo {
    PhaseModulation,
    AmplitudeModulation,
    #[default]
    SignalMix,
    SynchronizedBySubOsc,
    FrequencyModulation,
}
