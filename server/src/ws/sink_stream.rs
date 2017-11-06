use futures::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};

pub(super) struct SinkStream<T> {
    pub sink: UnboundedSender<T>,
    pub stream: Option<UnboundedReceiver<T>>,
}

impl<T> SinkStream<T> {
    pub fn new() -> Self {
        let (sink, stream) = mpsc::unbounded();
        Self {
            sink: sink,
            stream: Some(stream),
        }
    }

    pub fn take_stream(&mut self) -> UnboundedReceiver<T> {
        self.stream.take().expect("Stream is already taken by other")
    }
}

