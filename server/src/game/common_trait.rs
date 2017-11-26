use common::*;
use slog::Logger;
use serde_json;
use serde::de::DeserializeOwned;
use serde::Serialize;
use ws::OwnedMessage;

pub trait Loggable {
    fn get_logger(&self) -> &Logger;
}

pub trait RawMessageSink {
    fn proc_raw_message(&mut self, Id, msg: String);
    fn user_disconnect(&mut self, Id);
}

pub trait Service: Loggable {
    type Message: DeserializeOwned;
    fn proc_message(&mut self, Id, msg: Self::Message);
    fn user_disconnect(&mut self, Id) {}
}


impl<T: Service> RawMessageSink for T {
    fn proc_raw_message(&mut self, id: Id, msg: String) {
        if let Ok(msg) = serde_json::from_str(&msg) {
            self.proc_message(id, msg);
        } else {
            error!(self.get_logger(), "parse message failed"; "raw_str" => msg)
        }
    }

    fn user_disconnect(&mut self, id: Id) {
        Service::user_disconnect(self, id);
    }
}

pub trait RawService: RawMessageSink + Loggable {
}
pub type WsSender = UnboundedSender<(Id, OwnedMessage)>;

pub trait OutputSender {
    type Output: Serialize;

    fn get_send_sink(&self) -> &WsSender;
    fn send(&self, id: Id, output: &Self::Output) {
        self.get_send_sink().unbounded_send(
            (id, OwnedMessage::Text(serde_json::to_string(output).unwrap()))
        ).unwrap();
    }
}


