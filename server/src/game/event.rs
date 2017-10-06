use super::*;

pub enum Event {
    UserMessage(Id, UserMessage),
}

#[derive(Deserialize, Debug)]
pub enum UserMessage {
    Join,
}


