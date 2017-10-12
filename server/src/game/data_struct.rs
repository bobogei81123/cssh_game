use common::*;
use super::point::Point;
use super::state::UserState;

#[derive(Debug, Serialize, Clone)]
pub struct Health {
    pub max: f64,
    pub value: f64,
}

impl Health {
    pub fn add(&mut self, v: f64) {
        self.value = f64::min(self.max, self.value + v);
    }

    pub fn sub(&mut self, v: f64) {
        self.value = f64::max(0f64, self.value - v);
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct User {
    pub id: Id,
    pub pos: Point,
    pub health: Health,

    #[serde(skip_serializing)]
    pub assigned_problem: Option<usize>,
    #[serde(skip_serializing)]
    pub state: UserState,
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

#[derive(Serialize, Clone, Debug)]
pub struct Damage {
    pub target: Id,
    pub value: f64,
    pub health_after: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct FireOut {
    pub id: Id,
    pub fire: Fire,
    pub damage: Option<Damage>,
}
