use common::*;
use super::game::UserSend;
use super::game::Runner;
use std::boxed::FnBox;

#[allow(dead_code)]
pub enum Event {
    Connect(Id),
    Disconnect(Id),
    UserSend(Id, UserSend),
    Timeout(Box<FnBox(&mut Runner) -> () + Send>),
}
