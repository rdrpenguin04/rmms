fn main() {
    lmms2_core::audio::spawn_engine(|x| {
        eprintln!("audio error: {}", x);
    }).expect("No audio");
}