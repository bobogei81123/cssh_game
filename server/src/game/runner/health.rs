#[derive(Debug, Serialize, Clone)]
pub struct Health {
    pub max: f64,
    pub value: f64,
}

impl Health {
    pub fn add(&mut self, v: f64) {
        self.value = self.max.min(self.value + v);
    }

    pub fn sub(&mut self, v: f64) {
        self.value = (0f64).max(self.value - v);
    }
}
