use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::RwLock;

use tokio_core::reactor::Core;
use websocket::async::Server;
use futures::{Future, Stream, Sink, future};

use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;

fn consume_future_result<F, I, E>(f: F) -> impl Future<Item=(), Error=()>
    where F: Future<Item=I, Error=E> + 'static {
    f.map_err(|_| ()).map(|_| ())
}

pub fn start_websocket() {
    let mut core = Core::new().expect("Failed to create Tokio event loop");
    let handle = core.handle();
    let remote = core.remote();
    let server = Server::bind("0.0.0.0:3210", &handle).expect("Failed to create server");

    let connections = RwLock::new(HashMap::new());

    let connection_future = server.incoming()
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            if !upgrade.protocols().contains(&"rust-websocket".to_owned()) {
                handle.spawn(consume_future_result(upgrade.reject()));
                return Ok(());
            }

            let fut = upgrade
                .use_protocol("rust-websocket")
                .accept()
                .and_then(|(s, _)| {
                    let con_ref = &connections;
                    s.send(Message::text("Hao123").into())
                });

            handle.spawn(consume_future_result(fut));

            Ok(())
        });

    core.run(connection_future).unwrap();
}
