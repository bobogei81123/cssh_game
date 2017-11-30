#[derive(Serialize)]
pub struct Health {
    pub max: f64,
    pub value: f64,
}

impl Default for Health {
    fn default() -> Health {
        Health {
            max: 100.,
            value: 100.,
        }
    }
}

impl Health {
    #[allow(dead_code)]
    pub fn add(&mut self, v: f64) {
        self.value = f64::min(self.max, self.value + v);
    }

    pub fn sub(&mut self, v: f64) {
        self.value = f64::max(0f64, self.value - v);
    }
}

