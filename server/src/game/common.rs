pub extern crate serde;
pub extern crate serde_json;

pub use common::*;
pub use std::rc::Rc;
pub use std::cell::RefCell;
pub use std::collections::HashMap;
pub use slog::Logger;
pub use ws::OwnedMessage;

pub use self::serde::de::DeserializeOwned;
pub use self::serde::Serialize;

use std::borrow::Borrow;

pub use super::point::Point;
pub use super::utils;

pub trait Loggable {
    fn logger(&self) -> &Logger;
}

pub trait RawMessageSink {
    fn proc_raw_message(&mut self, Id, msg: String);
}

pub trait MessageSink {
    type Message: DeserializeOwned;
    fn proc_message(&mut self, Id, msg: Self::Message);
}

impl<T: MessageSink + Loggable> RawMessageSink for T {
    fn proc_raw_message(&mut self, id: Id, msg: String) {
        if let Ok(msg) = serde_json::from_str(&msg) {
            self.proc_message(id, msg);
        } else {
            error!(self.logger(), "parse message failed"; "raw_str" => msg)
        }
    }
}

pub trait UserEventListener {
    fn user_disconnect(&mut self, Id) {}
}

pub trait RawService: RawMessageSink + UserEventListener { }
pub type RcRawService = Rc<RefCell<RawService>>;
pub type SinkMap = HashMap<Id, RcRawService>;

pub type WsSender = UnboundedSender<(Id, OwnedMessage)>;


pub trait OutputSender {
    type Output: Serialize;

    fn get_send_sink(&self) -> &WsSender;
    fn send(&self, id: Id, output: &Self::Output) {
        self.get_send_sink().unbounded_send(
            (id, OwnedMessage::Text(serde_json::to_string(output).unwrap()))
        ).unwrap();
    }

    fn send_many<T, I>(&self, ids: T, output: &Self::Output)
        where T: IntoIterator<Item=I>, I: Borrow<Id> {
        let msg = OwnedMessage::Text(serde_json::to_string(output).unwrap());
        for id in ids {
            self.get_send_sink().unbounded_send((*id.borrow(), msg.clone())).unwrap();
        }
    }
}

#[derive(Clone)]
pub struct Common {
    pub handle: Handle,
    pub ws_sender: WsSender,
    pub logger: Logger,
}
