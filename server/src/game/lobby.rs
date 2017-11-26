use common::*;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use serde_json;

use super::common_trait::*;
use super::room::Room;

type RcRawService = Rc<RefCell<RawService>>;

#[derive(Clone)]
pub struct User(pub Id, pub String);

pub struct Lobby {
    output_sink: WsSender,
    logger: Logger,
    sink_map: HashMap<Id, RcRawService>,
    users: HashMap<Id, User>,
    _default_room: Rc<RefCell<Room>>,
}

impl Loggable for Lobby {
    fn get_logger(&self) -> &Logger { &self.logger }
}

#[derive(Deserialize, Debug)]
pub enum Message {
    RequestInitial(String),
    Join,
}

#[derive(Serialize, Debug)]
pub enum Output {
    Initial(Id),
}

impl OutputSender for Lobby {
    type Output = Output;

    fn get_send_sink(&self) -> &WsSender {
        &self.output_sink
    }
}

impl Lobby {
    pub fn new(output_sink: WsSender, logger: Logger) -> Self {
        Self {
            output_sink: output_sink.clone(),
            logger: logger.clone(),
            sink_map: HashMap::new(),
            users: HashMap::new(),

            _default_room: Rc::new(RefCell::new(Room::new(
                        output_sink, logger.new(o!("who" => "Room"))))),
        }
    }

    pub fn proc_raw_message(&mut self, id: Id, msg: String) {
        if let Some(sink) = self.sink_map.get(&id) {
            return sink.borrow_mut().proc_raw_message(id, msg);
        }
        RawService::proc_raw_message(self, id, msg)
    }
}

impl Service for Lobby {
    type Message = Message;

    fn proc_message(&mut self, id: Id, msg: Message) {
        match msg {
            Message::RequestInitial(name) => {
                self.send(id, &Output::Initial(id));
                info!(self.logger, "User connect"; "id" => id, "name" => &name);
                self.users.insert(id, User(id, name));
            }
            Message::Join => {
                let user = self.users.get(&id);
                if let Some(user) = user {
                    let sink = self.sink_map.insert(id, self._default_room.clone());
                    self._default_room.borrow_mut().user_entering((*user).clone());
                }
            }
        }
    }

    fn user_disconnect(&mut self, _id: Id) {}
}

impl RawService for Lobby {}
