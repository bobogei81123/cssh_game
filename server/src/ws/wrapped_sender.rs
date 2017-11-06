use futures::{Poll, Sink, StartSend};
use futures::sync::mpsc::{UnboundedSender, SendError};
use websocket::message::OwnedMessage;
use futures::AsyncSink;

use common::*;

#[derive(Debug, Clone)]
pub struct WrappedSender(pub UnboundedSender<(Id, OwnedMessage)>);

impl WrappedSender {
    pub fn unbounded_send(&self, (id, msg): (Id, String))
        -> Result<(), SendError<(Id, OwnedMessage)>> {
        self.0.unbounded_send((id, OwnedMessage::Text(msg)))
    }
}

impl Sink for WrappedSender {
    type SinkItem = (Id, String);
    type SinkError = <UnboundedSender<(Id, OwnedMessage)> as Sink>::SinkError;

    fn start_send(&mut self, (id, msg): Self::SinkItem)
        -> StartSend<Self::SinkItem, Self::SinkError> {

        self.0.start_send((id, OwnedMessage::Text(msg))).map(|x| {
            match x {
                AsyncSink::Ready => AsyncSink::Ready,
                AsyncSink::NotReady((id, _msg)) => {
                    if let OwnedMessage::Text(msg) = _msg {
                        AsyncSink::NotReady((id, msg))
                    } else { unreachable!() }
                }
            }
        })

    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.0.poll_complete()
    }
}

