extern crate websocket;
extern crate chrono;
extern crate byteorder;

mod counter;
mod sink_stream;
pub mod wrapped_sender;

use common::*;

use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use std::rc::Rc;
use std::cell::RefCell;
use std::thread;

use self::chrono::Utc;
use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use tokio_core::reactor::{Core, Handle, Remote};
use tokio_core::net::TcpStream;

use futures::future::{self, Either, Loop};
use futures::stream::{SplitSink, SplitStream};

use futures_cpupool::CpuPool;

use self::websocket::async::{Client, Server};
use self::websocket::message::OwnedMessage;
use self::websocket::server::InvalidConnection;

use self::counter::Counter;
use self::sink_stream::SinkStream;
use self::wrapped_sender::WrappedSender;

type MyClient = SplitSink<Client<TcpStream>>;

pub struct WsServer {
    id_counter: Rc<RefCell<Counter>>,
    connections: Arc<RwLock<HashMap<Id, MyClient>>>,
    init_channel: SinkStream<(Id, SplitStream<Client<TcpStream>>)>,
    send_channel: SinkStream<(Id, OwnedMessage)>,
    event_channel: SinkStream<WsEvent>,
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
            init_channel: SinkStream::new(),
            send_channel: SinkStream::new(),
            event_channel: SinkStream::new(),
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

    fn make_connection_future<'a>(
        &self,
        handle: &Handle,
    ) -> impl Future<Item = (), Error = ()> + 'a {
        let server =
            Server::bind("0.0.0.0:3210", &handle).expect("Failed to create websocket server");


        server
            .incoming()
            .map_err(|InvalidConnection { error, .. }| error)
            .for_each({
                let connections = self.connections.clone();
                let id_counter = self.id_counter.clone();
                let init_sink = self.init_channel.sink.clone();
                let event_sink = self.event_channel.sink.clone();
                let logger = self.logger.clone();
                let handle = handle.clone();

                move |(upgrade, addr)| {
                    let handle = handle.clone();

                    if !upgrade.protocols().contains(&"rust-websocket".to_owned()) {
                        Self::spawn(&handle, upgrade.reject());
                        return Ok(());
                    }

                    let connect_fun = capture!(
                        connections, id_counter, init_sink, handle, logger =>
                        move |(client, _): (Client<TcpStream>, _)| {
                            info!(logger, "A client connected"; "ip" => addr.to_string());
                            let id = id_counter.borrow_mut().next().unwrap();
                            let (sink, stream) = client.split();

                            Self::spawn(&handle, init_sink.send((id, stream)));
                            connections.write().unwrap().insert(id, sink);

                            Ok(id)
                        }
                    );

                    let send_fun = capture!(
                        event_sink, handle =>
                        move |id| {
                            Self::spawn(&handle, event_sink.send(WsEvent::Connect(id)));
                            Ok(())
                        }
                    );

                    let combined_future = upgrade
                        .use_protocol("rust-websocket")
                        .accept()
                        .and_then(connect_fun)
                        .and_then(send_fun);
                    Self::spawn(&handle, combined_future);

                    Ok(())
                }
            })
            .map_err(|_| ())
    }

    fn make_send_future(
        &mut self,
        pool: &CpuPool,
        remote: &Remote,
    ) -> impl Future<Item = (), Error = ()> {
        pool.spawn_fn({
            let send_stream = self.send_channel.take_stream();
            let event_sink = self.event_channel.sink.clone();
            let connections = self.connections.clone();
            let logger = self.logger.clone();
            let remote = remote.clone();

            move || {
                send_stream
                    .and_then(capture!(connections =>
                        move |(id, msg)| {
                    let mut conn = connections.write().unwrap();
                    let conn = conn.remove(&id);

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
            }
        })
    }

    fn make_init_future(
        &mut self,
        pool: &CpuPool,
        remote: &Remote,
    ) -> impl Future<Item = (), Error = ()> {
        pool.spawn_fn({
            let init_stream = self.init_channel.take_stream();
            let remote = remote.clone();
            let event_sink = self.event_channel.sink.clone();
            let connections = self.connections.clone();
            let pool = pool.clone();

            move || {
                init_stream.for_each(move |(id, stream)| {
                    Self::spawn_remote(
                        &remote,
                        pool.spawn_fn(
                            capture!(event_sink, remote, connections => move || {

                            let message_fun = move |msg| {
                                match msg {
                                    OwnedMessage::Text(text) => {
                                        Self::spawn_remote(
                                            &remote,
                                            event_sink.clone().send(WsEvent::Message(id, text))
                                        );
                                    }
                                    OwnedMessage::Pong(vec) => {
                                        let diff = get_diff(vec);

                                        Self::spawn_remote(
                                            &remote,
                                            event_sink.clone().send(WsEvent::Ping(id, diff))
                                        );
                                    }
                                    OwnedMessage::Close(_) => {
                                        connections.write().unwrap().remove(&id);
                                        Self::spawn_remote(
                                            &remote,
                                            event_sink
                                                .clone()
                                                .send(WsEvent::Disconnect(id)));
                                    }
                                    _ => { println!("{:?}", msg); }
                                }
                                Ok(())
                            };

                            consume_result!(stream.for_each(message_fun))
                        }),
                        ),
                    );
                    Ok(())
                })
            }
        })
    }

    fn make_ping_future(
        &mut self,
        pool: &CpuPool,
        remote: &Remote,
    ) -> impl Future<Item = (), Error = ()> {
        pool.spawn(future::loop_fn((), {
            let remote = remote.clone();
            let connections = self.connections.clone();
            let send_sink = self.send_channel.sink.clone();

            move |()| {
                {
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
                }
                thread::sleep(Duration::from_secs(1));
                Ok::<Loop<(), ()>, ()>(Loop::Continue(()))
            }
        }))
    }

    //fn make_event_future(&mut self, mut runner: Runner)
    //-> impl Future<Item=(), Error=()> {
    //self.event_channel.take_stream().for_each(move |event| {
    //runner.proc_event(event);
    //Ok(())
    //})
    //}


    pub fn get_future<'a>(&mut self, core: &Core) -> impl Future<Item = (), Error = ()> + 'a {
        let handle = core.handle();
        let remote = core.remote();
        let pool = CpuPool::new_num_cpus();

        //let mut runner = Runner::new(
        //handle.clone(),
        //self.send_channel.sink.clone(),
        //self.event_channel.sink.clone(),
        //);

        //runner.init();

        Future::join4(
            self.make_connection_future(&handle),
            self.make_send_future(&pool, &remote),
            self.make_init_future(&pool, &remote),
            self.make_ping_future(&pool, &remote),
            //self.make_event_future(runner),
        ).map(|_| ())
            .map_err(|_| ())
    }

    pub fn take(&mut self) -> (UnboundedReceiver<WsEvent>, WrappedSender) {
        (
            self.event_channel.take_stream(),
            WrappedSender(self.send_channel.sink.clone()),
        )
    }
}

fn get_diff(vec: Vec<u8>) -> i64 {
    let mut vec = vec.as_slice();
    let sec = vec.read_i64::<BigEndian>().unwrap();
    let milli_sec = vec.read_u32::<BigEndian>().unwrap();

    let now = Utc::now();
    let diff = 1000 * (now.timestamp() - sec)
        + ((now.timestamp_subsec_millis() as i64 - milli_sec as i64));

    return diff;
}
