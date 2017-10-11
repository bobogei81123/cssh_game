use common::*;
use super::point::Point;

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

#[derive(Debug, Serialize, Clone)]
pub struct User {
    pub id: Id,
    pub pos: Point,
    pub health: Health,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Fire {
    pub pos: Point,
    pub angle: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct Initial {
    pub your_id: Id,
}
