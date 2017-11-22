extern crate byteorder;
extern crate chrono;
extern crate websocket;

mod counter;
mod sink_stream;
pub mod wrapped_sender;

use common::*;

use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use std::rc::Rc;
use std::cell::RefCell;

use self::chrono::Utc;
use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use tokio_core::reactor::{Core, Handle, Remote, Interval};
use tokio_core::net::TcpStream;

use futures::future::{self, Either};
use futures::stream::{SplitSink, SplitStream};
use futures::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use futures_cpupool::CpuPool;

use self::websocket::async::{Client, Server};
pub use self::websocket::message::OwnedMessage;
use self::websocket::server::InvalidConnection;

use self::counter::Counter;

type ClientSink = SplitSink<Client<TcpStream>>; 
type ClientStream = SplitStream<Client<TcpStream>>; 
type MyClient = ClientSink;

pub struct WsServer {
    id_counter: Rc<RefCell<Counter>>,
    connections: Arc<RwLock<HashMap<Id, MyClient>>>,
    pool: CpuPool,
    logger: Logger,
}

pub enum WsEvent {
    Connect(Id),
    Message(Id, String),
    Ping(Id, i64),
    Disconnect(Id),
}

impl WsServer {
    pub fn new(logger: Logger) -> Self {
        Self {
            id_counter: Rc::new(RefCell::new(Counter::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            pool: CpuPool::new_num_cpus(),
            logger: logger,
        }
    }

    fn spawn<F, I, E>(handle: &Handle, future: F)
    where
        F: Future<Item = I, Error = E> + 'static,
    {
        handle.spawn(future.map(|_| ()).map_err(|_| ()));
    }

    fn spawn_remote<F, I, E>(remote: &Remote, future: F)
    where
        F: Future<Item = I, Error = E> + Send + 'static,
    {
        remote.spawn(|_| future.map(|_| ()).map_err(|_| ()));
    }

    fn make_receive_future(
        id: Id,
        stream: ClientStream,
        pool: CpuPool,
        event_sink: UnboundedSender<WsEvent>,
        connections: Arc<RwLock<HashMap<Id, MyClient>>>,
    ) -> impl Future<Item = (), Error = ()> {

        pool.spawn(
            consume_result!(stream.for_each(move |msg| {
                match msg {
                    OwnedMessage::Text(text) => {
                        event_sink.unbounded_send(WsEvent::Message(id, text));
                    }
                    OwnedMessage::Pong(ref vec) => {
                        let diff = get_diff(vec);
                        event_sink.unbounded_send(WsEvent::Ping(id, diff));
                    }
                    OwnedMessage::Close(_) => {
                        connections.write().unwrap().remove(&id);
                        event_sink.unbounded_send(WsEvent::Disconnect(id));
                    }
                    _ => { }
                }
                Ok(())
            }))
        )
    }

    #[allow(needless_pass_by_value)]
    fn spawn_connection_future<'a>(
        &self,
        core: &Core,
        //init_sink: UnboundedSender<(Id, ClientStream)>,
        event_sink: UnboundedSender<WsEvent>,
    ) {

        let handle = core.handle();
        let pool = self.pool.clone();
        let server = Server::bind("0.0.0.0:3210", &handle)
            .expect("Failed to create websocket server");

        handle.spawn(server
            .incoming()
            .map_err(|InvalidConnection { error, .. }| error)
            .for_each({
                let connections = self.connections.clone();
                let id_counter = self.id_counter.clone();
                let logger = self.logger.clone();
                let handle = handle.clone();

                move |(upgrade, addr)| {
                    let handle = handle.clone();

                    if !upgrade.protocols().contains(&"rust-websocket".to_owned()) {
                        Self::spawn(&handle, upgrade.reject());
                        return Ok(());
                    }

                    let combined_future = upgrade
                        .use_protocol("rust-websocket")
                        .accept()
                        .and_then(capture!(
                            connections, id_counter, handle, logger, pool, event_sink =>
                            move |(client, _): (Client<TcpStream>, _)| {
                                info!(logger, "A client connected"; "ip" => addr.to_string());
                                let id = id_counter.borrow_mut().next().unwrap();
                                let (sink, stream) = client.split();

                                connections.write().unwrap().insert(id, sink);
                                handle.spawn(
                                    Self::make_receive_future(
                                        id, stream, pool.clone(), event_sink.clone(), connections.clone())
                                );
                                event_sink.unbounded_send(WsEvent::Connect(id));
                                Ok(())
                            }
                        ));
                    Self::spawn(&handle, combined_future);

                    Ok(())
                }
            })
            .map_err(|_| ())
        );
    }

    fn spawn_send_future(
        &mut self,
        core: &Core,
        send_stream: UnboundedReceiver<(Id, OwnedMessage)>,
        event_sink: UnboundedSender<WsEvent>,
    ) {

        let remote = core.remote();
        let connections = self.connections.clone();
        let logger = self.logger.clone();

        core.handle().spawn(
            self.pool.spawn(
                send_stream
                    .and_then(capture!(connections => move |(id, msg)| {
                        let conn = connections.write().unwrap().remove(&id);

                        match conn {
                            Some(sink) => {
                                Either::A(sink.send(msg)
                                    .map(move |sink| Some((sink, id)))
                                    .or_else(
                                        capture!(connections, event_sink, remote => move |_| {
                                            connections.write().unwrap().remove(&id);
                                            Self::spawn_remote(
                                                &remote,
                                                event_sink.send(WsEvent::Disconnect(id)));
                                            Ok(None)
                                        })
                                    ))
                            }
                            None => {
                                warn!(logger, "Try to send to dead client {}", id);
                                Either::B(future::ok(None))
                            }
                        }
                    }))
                    .for_each(capture!(connections => move |opt| {
                        if let Some((sink, id)) = opt {
                            connections.write().unwrap().insert(id, sink);
                        }
                        Ok(())
                    }))
            )
        );
    }

    fn spawn_ping_future(
        &mut self,
        core: &Core,
        send_sink: UnboundedSender<(Id, OwnedMessage)>,
    ) {
        let handle = core.handle();
        let remote = core.remote();

        core.handle().spawn(
            self.pool.spawn(
                consume_result!(
                    Interval::new(Duration::from_secs(1), &handle).unwrap()
                        .for_each({
                        let connections = self.connections.clone();

                        move |()| {
                            let connections = connections.write().unwrap();
                            for (id, _) in connections.iter() {
                                let current_time = Utc::now();
                                let mut vec = vec![];
                                vec.write_i64::<BigEndian>(current_time.timestamp())
                                    .unwrap();
                                vec.write_u32::<BigEndian>(current_time.timestamp_subsec_millis())
                                    .unwrap();
                                Self::spawn_remote(
                                    &remote,
                                    send_sink.clone().send((*id, OwnedMessage::Ping(vec))),
                                    );
                            }
                            Ok(())
                        }
                    })
                )
            )
        );
    }

    pub fn spawn_futures<'a>(
        &mut self,
        core: &Core,
    ) -> (UnboundedSender<(Id, OwnedMessage)>, UnboundedReceiver<WsEvent>) {

        let (event_sink, event_stream) = mpsc::unbounded::<WsEvent>();
        let (send_sink, send_stream) = mpsc::unbounded::<(Id, OwnedMessage)>();

        self.spawn_connection_future(core, event_sink.clone());
        self.spawn_send_future(core, send_stream, event_sink.clone());
        self.spawn_ping_future(core, send_sink.clone());

        (send_sink, event_stream)
    }
}

fn get_diff(mut vec: &[u8]) -> i64 {
    let sec = vec.read_i64::<BigEndian>().unwrap();
    let milli_sec = vec.read_u32::<BigEndian>().unwrap();

    let now = Utc::now();

    #[allow(cast_lossless)]
    {
        1000 * (now.timestamp() - sec) + (now.timestamp_subsec_millis() as i64 - milli_sec as i64)
    }
}
