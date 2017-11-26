mod common_trait;
mod lobby;
mod room;
//mod point;
//mod data_struct;
//mod user_send;
//mod state;
//mod output;
//mod constant;
//mod problem;

use common::*;



use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;

#[allow(unused_imports)]
use tokio_core::reactor::Timeout;

use ws::WsEvent;
use ws::OwnedMessage;

#[allow(unused_imports)]
use event::Event;

use ws::WsServer;
use logger;

pub use self::common_trait::*;
use self::lobby::Lobby;

#[derive(Clone)]
struct GameServerSender(WsSender);

#[derive(Serialize)]
enum GameServerOutput {
    Ping(i64),
}

impl OutputSender for GameServerSender {
    type Output = GameServerOutput;
    fn get_send_sink(&self) -> &WsSender { &self.0 }
}


pub struct GameServer {
    logger: Logger,
}


impl GameServer {
    pub fn new() -> Self {
        let logger = logger::make_logger();
        Self { 
            logger: logger.clone(),
        }
    } 
    pub fn spawn_ws_event(
        &mut self,
        core: &Core,
        ws_event_stream: UnboundedReceiver<WsEvent>,
        send_sink: WsSender,
    ) {

        let handle = core.handle();
        let logger = self.logger.clone();
        let mut lobby = //Rc::new(RefCell::new(
                Lobby::new(
                    send_sink.clone(),
                    logger.new(o!("who" => "Lobby")),
                );
            //));

        let server_sender = GameServerSender(send_sink);

        handle.spawn(
            ws_event_stream
            .for_each(move |event| {
                match event {
                    WsEvent::Connect(id) => {
                        //sink_map.borrow_mut().insert(id, lobby.clone());
                    },
                    WsEvent::Disconnect(id) => {
                        //sink_map.borrow_mut().remove(&id);
                    },
                    WsEvent::Message(id, msg) => {
                        lobby.proc_raw_message(id, msg);
                    }
                    WsEvent::Ping(id, diff) => {
                        server_sender.send(id, &GameServerOutput::Ping(diff))
                    }
                }
                Ok(())
            })
        )
    }

    pub fn start(mut self) {
        let mut core = Core::new().expect("Failed to create event loop");
        let mut ws_server = WsServer::new(self.logger.new(o!("who" => "WS")));
        let (sink, stream) = ws_server.spawn_futures(&core);
        self.spawn_ws_event(&core, stream, sink);
        core.run(futures::empty::<(), ()>()).unwrap();
    }
}
