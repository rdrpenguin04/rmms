use std::ops::RangeInclusive;

pub struct Model<T: Copy> {
    value: T,
    range: Option<RangeInclusive<T>>,
    display_name: String,
    full_display_name: Option<String>,
    was_changed: bool,
    map: Box<dyn Fn(T) -> T>,
    rev_map: Box<dyn Fn(T) -> T>,
}

impl<T: Copy> Model<T> {
    pub fn get_value(&self) -> T {
        (self.map)(self.value)
    }

    pub fn set_value(&mut self, value: T) {
        self.value = (self.rev_map)(value);
        self.was_changed = true;
    }
    
    pub const fn get_unmapped_value(&self) -> T {
        self.value
    }

    pub fn set_unmapped_value(&mut self, value: T) {
        self.value = value;
        self.was_changed = true;
    }

    pub fn map(&self, value: T) -> T {
        (self.map)(value)
    }

    pub fn reverse_map(&self, value: T) -> T {
        (self.rev_map)(value)
    }

    pub fn range(&self) -> Option<RangeInclusive<T>> {
        self.range.clone()
    }

    pub fn mapped_range(&self) -> Option<RangeInclusive<T>> {
        self.range
            .clone()
            .map(|x| RangeInclusive::new(*x.start(), *x.end()))
    }

    pub fn set_map(&mut self, map: Box<dyn Fn(T) -> T>, rev_map: Box<dyn Fn(T) -> T>) {
        self.map = map;
        self.rev_map = rev_map;
    }

    pub const fn display_name(&self) -> &String {
        &self.display_name
    }

    pub const fn full_display_name(&self) -> &Option<String> {
        &self.full_display_name
    }

    pub fn set_display_name(&mut self, display_name: String) {
        self.display_name = display_name;
    }

    pub fn set_full_display_name(&mut self, full_display_name: Option<String>) {
        self.full_display_name = full_display_name;
    }
}
