use common::*;

use std::rc::Weak;

use super::common::*;
use super::room::Room;

#[derive(Clone)]
pub struct User(pub Id, pub String);

pub struct Lobby {
    common: Common,
    sink_map: Rc<RefCell<SinkMap>>,
    users: HashMap<Id, User>,
    _default_room: Weak<RefCell<Room>>,
}

impl_loggable!(Lobby);

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
        &self.common.ws_sender
    }
}

impl Lobby {
    pub fn new(
        handle: Handle,
        output_sink: WsSender,
        logger: Logger,
    ) -> Self {
        let sink_map = Rc::new(RefCell::new(HashMap::new()));
        let common = Common {
                handle: handle.clone(),
                ws_sender: output_sink.clone(),
                logger: logger.clone(),
            };
        Self {
            common: common.clone(),
            sink_map: sink_map.clone(),
            users: HashMap::new(),

            _default_room: Weak::new(),
        }
    }

    pub fn proc_raw_message(&mut self, id: Id, msg: String) {
        let sink = do catch {
            Some(self.sink_map.borrow().get(&id)?.clone())
        };

        if let Some(sink) = sink {
            sink.borrow_mut().proc_raw_message(id, msg);
        } else {
            RawMessageSink::proc_raw_message(self, id, msg);
        }
    }

    pub fn user_disconnect(&mut self, id: Id) {
        info!(self.logger(), "User disconnect"; "id" => id);
        if let Some(sink) = self.sink_map.borrow_mut().remove(&id) {
            sink.borrow_mut().user_disconnect(id);
        }
    }
}

impl MessageSink for Lobby {
    type Message = Message;

    fn proc_message(&mut self, id: Id, msg: Message) {
        match msg {
            Message::RequestInitial(name) => {
                self.send(id, &Output::Initial(id));
                info!(self.logger(), "User connect"; "id" => id, "name" => &name);
                self.users.insert(id, User(id, name));
            }
            Message::Join => {
                let user = self.users.get(&id);
                if let Some(user) = user {
                    println!("{}", self._default_room.upgrade().map(|x| Rc::strong_count(&x)).unwrap_or(0));
                    let room = self._default_room.upgrade().unwrap_or(
                        Rc::new(RefCell::new(Room::new(
                                Common {
                                    logger: self.logger().new(o!("who" => "Room")),
                                    ..self.common.clone()
                            }, self.sink_map.clone()
                        ))) 
                    );
                    self._default_room = Rc::downgrade(&room);
                    let _sink = self.sink_map.borrow_mut().insert(id, room.clone());
                    room.borrow_mut().user_entering((*user).clone());
                }
            }
        }
    }
}
