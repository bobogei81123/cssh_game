use common::*;

pub struct Counter {
    count: Id,
}

impl Counter {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Iterator for Counter {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count != Self::Item::max_value() {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}
