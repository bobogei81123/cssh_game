use super::*;

pub enum Event {
    Connect(Id),
    Disconnect(Id),
    UserMessage(Id, UserMessage),
}

#[derive(Deserialize, Debug)]
pub enum UserMessage {
    Join,
}


