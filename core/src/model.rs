use std::ops::RangeInclusive;

pub struct Model<T: Copy> {
    value: T,
    range: Option<RangeInclusive<T>>,
    display_name: String,
    full_display_name: String,
    was_changed: bool,
    mapping: Box<dyn Fn(T) -> T>,
}
