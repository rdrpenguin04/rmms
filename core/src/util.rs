#[derive(Clone, Copy, Debug, Default)]
pub struct Stereo<T> {
    pub l: T,
    pub r: T,
}
