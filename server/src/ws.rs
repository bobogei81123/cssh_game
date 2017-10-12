use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use std::rc::Rc;
use std::cell::RefCell;
use std::thread;

use chrono::Utc;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

use tokio_core::reactor::{Core, Handle, Remote};
use websocket::async::Server;
use futures::{Future, Stream, Sink};
use futures::future::{self, Loop};
use futures::sync::mpsc;
use futures_cpupool::CpuPool;

use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;
use websocket::client::async::Client;

use serde_json;

use common::*;
use game::{Runner, UserSend};
use event::Event;


struct Counter {
    count: Id,
}

impl Counter {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl Iterator for Counter {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count != Self::Item::max_value() {
            self.count += 1;
            return Some(self.count);
        } else {
            return None;
        }
    }
}

type MyClient = SplitSink<Client<TcpStream>>;

use futures::stream::{SplitSink, SplitStream};
use tokio_core::net::TcpStream;
use futures::sync::mpsc::{UnboundedSender, UnboundedReceiver};

struct SinkStream<T> {
    sink: UnboundedSender<T>,
    stream: Option<UnboundedReceiver<T>>,
}

impl<T> SinkStream<T> {
    fn new() -> Self {
        let (sink, stream) = mpsc::unbounded();
        Self {
            sink: sink,
            stream: Some(stream),
        }
    }

    fn take_stream(&mut self) -> UnboundedReceiver<T> {
        self.stream.take().unwrap()
    }
}

struct MainServer {
    core: Core,
    id_counter: Rc<RefCell<Counter>>,
    connections: Arc<RwLock<HashMap<Id, Option<MyClient>>>>,
    init_channel: SinkStream<(Id, SplitStream<Client<TcpStream>>)>,
    send_channel: SinkStream<(Id, OwnedMessage)>,
    event_channel: SinkStream<Event>,
}

impl MainServer {
    fn new() -> Self {
        let core = Core::new().expect("Failed to create Tokio event loop");

        Self {
            core: core,
            id_counter: Rc::new(RefCell::new(Counter::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            init_channel: SinkStream::new(),
            send_channel: SinkStream::new(),
            event_channel: SinkStream::new(),
        }
    }

    fn spawn<F, I, E>(handle: &Handle, future: F)
        where F: Future<Item = I, Error = E> + 'static {
        handle.spawn(future.map(|_| ()).map_err(|_| ()));
    }

    fn spawn_remote<F, I, E>(remote: &Remote, future: F)
        where F: Future<Item = I, Error = E> + Send + 'static {
        remote.spawn(|_| future.map(|_| ()).map_err(|_| ()));
    }

    fn make_connection_future<'a>(&self, handle: &Handle)
        -> impl Future<Item=(), Error=()> + 'a {
        let server = Server::bind("0.0.0.0:3210", &handle)
            .expect("Failed to create websocket server");


        server.incoming()
            .map_err(|InvalidConnection { error, .. }| error)
            .for_each({
                let connections = self.connections.clone();
                let id_counter = self.id_counter.clone();
                let init_sink = self.init_channel.sink.clone();
                let event_sink = self.event_channel.sink.clone();
                let handle = handle.clone();

                move |(upgrade, addr)| {

                    let handle = handle.clone();

                    if !upgrade.protocols().contains(&"rust-websocket".to_owned()) {
                        Self::spawn(&handle, upgrade.reject());
                        return Ok(());
                    }

                    let connect_fun = capture!(
                        connections, id_counter, init_sink, handle =>
                        
                        |(client, _)| {

                            info!(logger, "A client connected"; "ip" => addr.to_string());
                            let id = id_counter.borrow_mut().next().unwrap();
                            let client: Client<TcpStream> = client;
                            let (sink, stream) = client.split();
                            
                            Self::spawn(&handle, init_sink.send((id, stream)));
                            connections.write().unwrap().insert(id, Some(sink));

                            Ok(id)
                        }
                    );

                    let send_fun = capture!(
                        event_sink, handle =>

                        |id| {
                            Self::spawn(&handle, event_sink.send(Event::Connect(id)));
                            Ok(())
                        }
                    );

                    let combined_future = upgrade
                        .use_protocol("rust-websocket")
                        .accept()
                        .and_then(connect_fun)
                        .and_then(send_fun)
                        ;
                    Self::spawn(&handle, combined_future);

                    Ok(())
                }
            }).map_err(|_| ())
    }

    fn make_send_future(&mut self, pool: &CpuPool, remote: &Remote)
        -> impl Future<Item=(), Error=()> {

        pool.spawn_fn({
            let send_stream = self.send_channel.take_stream();
            let send_sink = self.send_channel.sink.clone();
            let event_sink = self.event_channel.sink.clone();
            let connections = self.connections.clone();
            let remote = remote.clone();

            move || {

                send_stream.for_each(move |(id, msg): (Id, OwnedMessage)| {
                    let mut conn = connections.write().unwrap();
                    let conn = conn.get_mut(&id);

                    match conn {
                        Some(client) => {
                            let client = client.take();
                            let remote = remote.clone();

                            match client {
                                Some(sink) => {
                                    let _msg: OwnedMessage = msg.clone();
                                    let future = sink.send(msg)
                                        .and_then(
                                            capture!(connections => |sink| {
                                                let mut entry = connections.write().unwrap();
                                                let entry = entry.get_mut(&id).unwrap();
                                                *entry = Some(sink);
                                                Ok(())
                                            })
                                        ).map_err(
                                            capture!(connections, event_sink, remote => |_| {
                                                connections.write().unwrap().remove(&id);
                                                Self::spawn_remote(&remote,
                                                                   event_sink.send(Event::Disconnect(id)));
                                            })
                                        );
                                    Self::spawn_remote(&remote, future);
                                }
                                None => {
                                    Self::spawn_remote(&remote, send_sink.clone().send((id, msg)));
                                }
                            }
                        }
                        None => {
                            warn!(logger, "Try to send to dead client {}", id);
                        }
                    }
                    Ok(())
                })
            }
        })
    }

    fn make_init_future(&mut self, pool: &CpuPool, remote: &Remote)
        -> impl Future<Item=(), Error=()> {

        pool.spawn_fn({
            let init_stream = self.init_channel.take_stream();
            let remote = remote.clone();
            let event_sink = self.event_channel.sink.clone();
            let send_sink = self.send_channel.sink.clone();
            let connections = self.connections.clone();
            let pool = pool.clone();

            move || {
                init_stream.for_each(move |(id, stream)| {
                    Self::spawn_remote(&remote,
                        pool.spawn_fn(capture!(event_sink, send_sink, remote, connections => || {

                            let message_fun = move |msg| {
                                match msg {
                                    OwnedMessage::Text(text) => {
                                        let decode: Option<UserSend> = serde_json::from_str(&text).ok();
                                        if let Some(user_msg) = decode {
                                            Self::spawn_remote(&remote, 
                                                               event_sink.clone().send(Event::UserSend(id, user_msg)));
                                        } else {
                                            warn!(logger, "Parse failed."; "id" => id, "msg" => text);
                                        }
                                    }
                                    OwnedMessage::Pong(vec) => {
                                        let mut vec = vec.as_slice();
                                        let sec = vec.read_i64::<BigEndian>().unwrap();
                                        let milli_sec = vec.read_u32::<BigEndian>().unwrap();

                                        let now = Utc::now();
                                        let diff = 1000 * (now.timestamp() - sec)
                                            + ((now.timestamp_subsec_millis() as i64 - milli_sec as i64));
                                        Self::spawn_remote(
                                            &remote,
                                            send_sink.clone().send((
                                                id,
                                                OwnedMessage::Text(json!({ "ping": [diff] }).to_string())
                                            ))
                                        )
                                    }
                                    OwnedMessage::Close(_) => {
                                        connections.write().unwrap().remove(&id);
                                        Self::spawn_remote(&remote,
                                                           event_sink.clone().send(Event::Disconnect(id)));
                                    }
                                    _ => { println!("{:?}", msg); }
                                }
                                Ok(())
                            };

                            consume_result!(stream.for_each(message_fun))
                        }))
                    );
                    Ok(())
                })
            }
        })
    }

    fn make_ping_future(&mut self, pool: &CpuPool, remote: &Remote)
        -> impl Future<Item=(), Error=()> {

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
                        vec.write_i64::<BigEndian>(current_time.timestamp()).unwrap();
                        vec.write_u32::<BigEndian>(current_time.timestamp_subsec_millis()).unwrap();
                        Self::spawn_remote(&remote, send_sink.clone().send((*id, OwnedMessage::Ping(vec))));
                    }
                }
                thread::sleep(Duration::from_secs(1));
                Ok::<Loop<(), ()>, ()>(Loop::Continue(()))
            }
        }))
    }

    fn make_event_future(&mut self, mut runner: Runner) 
        -> impl Future<Item=(), Error=()> {
        self.event_channel.take_stream().for_each(move |event| {
            runner.proc_event(event);
            Ok(())
        })
    }
 

    fn start(mut self) {
        info!(logger, "Starting server...");
        let handle = self.core.handle();
        let remote = self.core.remote();
        let pool = CpuPool::new_num_cpus();

        let mut runner = Runner::new(
            handle.clone(), 
            self.send_channel.sink.clone(),
            self.event_channel.sink.clone(),
        );

        runner.init();

        let combined_handler = Future::join5(
            self.make_connection_future(&handle),
            self.make_send_future(&pool, &remote),
            self.make_init_future(&pool, &remote),
            self.make_ping_future(&pool, &remote),
            self.make_event_future(runner),
        );
        self.core.run(combined_handler).unwrap();
    }
}

pub fn start() {
    let server = MainServer::new();
    server.start();
}

