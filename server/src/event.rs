use common::*;
use super::game::UserSend;

pub enum Event {
    Connect(Id),
    Disconnect(Id),
    UserSend(Id, UserSend),
}

