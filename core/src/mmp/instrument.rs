pub mod organic;
pub mod tripleoscillator;

use serde::Deserialize;

use self::organic::Organic;
use self::tripleoscillator::TripleOscillator;

#[derive(Debug, Deserialize)]
#[serde(tag = "@name")]
#[serde(rename_all = "lowercase")]
pub enum Instrument {
    Organic {
        #[serde(rename = "organic")]
        inner: Organic,
    },
    TripleOscillator {
        #[serde(rename = "tripleoscillator")]
        inner: TripleOscillator,
    },
}

#[derive(Clone, Debug, Default)]
pub enum WaveType {
    #[default]
    Uninit,
    SineWave,
    TriangleWave,
    SawWave,
    SquareWave,
    MoogSawWave,
    ExponentialWave,
    WhiteNoise,
    UserDefinedWave(String),
}
