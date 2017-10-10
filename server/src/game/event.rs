use super::*;
use super::runner::point::Point;

pub enum Event {
    Connect(Id),
    Disconnect(Id),
    UserMessage(Id, UserMessage),
}

#[derive(Deserialize, Debug)]
pub enum UserMessage {
    Join,
    Fire(Fire),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Fire {
    pos: Point,
    angle: f64,
}

