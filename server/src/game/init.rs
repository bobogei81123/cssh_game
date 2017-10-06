use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::rc::Rc;
use std::cell::RefCell;

use tokio_core::reactor::Core;
use websocket::async::Server;
use futures::{Future, Stream, Sink};
use futures::sync::mpsc;
use futures_cpupool::CpuPool;

use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;

use serde_json;

use super::*;
use super::runner::Runner;
use super::event::*;

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

pub fn init() {
    let mut core = Core::new().expect("Failed to create Tokio event loop");
    let handle = core.handle();
    let remote = core.remote();
    let server = Server::bind("0.0.0.0:3210", &handle).expect("Failed to create server");

    let id_counter = Rc::new(RefCell::new(Counter::new()));
    let connections = Arc::new(RwLock::new(HashMap::new()));

    let (init_sink, init_stream) = mpsc::unbounded();
    let (send_sink, send_stream) = mpsc::unbounded();
    let (receive_sink, receive_stream) = mpsc::unbounded();

    let connection_future = server.incoming()
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(capture!( handle, connections, send_sink => |(upgrade, _)| {
            println!("A client connected...");
            if !upgrade.protocols().contains(&"rust-websocket".to_owned()) {
                handle.spawn(consume_result!(upgrade.reject()));
                return Ok(());
            }

            let fut = upgrade
                .use_protocol("rust-websocket")
                .accept()
                .and_then(capture!(connections, id_counter, init_sink, handle => |(s, _)| {
                    let id = id_counter.borrow_mut().next().unwrap();
                    let (sink, stream) = s.split();
                    handle.spawn(consume_result!(init_sink.send((id, stream))));
                    connections.write().unwrap().insert(id, sink);
                    Ok(id)
                }))
                .and_then(capture!(send_sink, handle => |id| {
                    handle.spawn(
                        consume_result!(
                            send_sink.clone().send((id, format!("Hello, your id is {}", id)))
                    ));
                    Ok(())
                }));

            handle.spawn(consume_result!(fut));

            Ok(())
        })).map_err(|_| ());

    let pool = CpuPool::new_num_cpus();
    let send_handler = pool.spawn_fn(capture!(remote, send_sink => || {
        send_stream.for_each(move |(id, msg)| {
            let _out = connections.write().unwrap().remove(&id);
            match _out {
                Some(out) => {
                    let _msg = msg.clone();
                    let fut = out.send(OwnedMessage::Text(msg))
                        .and_then(capture!(connections => |out| {
                            connections.write().unwrap().insert(id, out);
                            Ok(())
                        }));
                    remote.spawn(move |_| {
                        consume_result!(
                            fut,
                            move |_| println!("Send to {} error.", id),
                            move |_| println!("Send to {}: {}", id, _msg)
                        )
                    });
                }
                None => {
                    remote.spawn(capture!(send_sink =>
                                          |_| consume_result!(send_sink.send((id, msg)))));
                }
            }
            Ok(())
        })
    }));

    let init_handler = pool.spawn_fn(capture!( remote, receive_sink => || {
        init_stream.for_each(move |(id, stream)| {
            remote.spawn(capture!(remote, receive_sink => |_| {
                consume_result!(stream.for_each(move |msg| {
                    if let OwnedMessage::Text(text) = msg {
                        println!("Receive {}: {}", id, text);
                        let message: Option<UserMessage> = serde_json::from_str(&text).ok();
                        if let Some(msg) = message {
                            remote.spawn(capture!( receive_sink =>
                               |_| consume_result!(receive_sink.send(Event::UserMessage(id, msg)))
                            ));
                        }
                    }
                    Ok(())
                }))
            }));
            Ok(())
        })
    }));

    let mut game_runner = Runner::new(handle.clone(), send_sink);

    let main_handler = receive_stream.for_each(move |event| {
        game_runner.proc_event(event);
        Ok(())
    });

    let combined_handler = Future::join4(connection_future, send_handler, init_handler, main_handler);
    core.run(combined_handler).unwrap();
}

