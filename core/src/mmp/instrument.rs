pub mod tripleoscillator;

use serde::Deserialize;

use self::tripleoscillator::TripleOscillator;

#[derive(Debug, Deserialize)]
#[serde(tag = "@name")]
#[serde(rename_all = "lowercase")]
pub enum Instrument {
    TripleOscillator {
        #[serde(rename = "tripleoscillator")]
        inner: TripleOscillator,
    },
}
