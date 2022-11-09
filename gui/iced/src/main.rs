fn main() {
    rmms_core::audio::spawn_engine(|x| {
        eprintln!("audio error: {x}");
    })
    .expect("No audio");
}
