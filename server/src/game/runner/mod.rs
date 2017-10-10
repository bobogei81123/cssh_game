use std::collections::HashMap;
use super::*;
use super::state::GameState;
use super::event::*;
use futures::sync::mpsc::UnboundedSender;
use tokio_core::reactor::Handle;
use futures::{Future, Sink};
use rand::{self, Rng};
use serde_json;

use websocket::message::OwnedMessage;

pub mod point;
use self::point::Point;

mod health;
use self::health::Health;

const GAME_WIDTH: u32 = 800;
const GAME_HEIGHT: u32 = 600;

#[derive(Serialize, Clone)]
struct User {
    id: Id,
    pos: Point,
    health: Health,
}

pub struct Runner {
    game_state: GameState,
    handle: Handle,
    output_sink: UnboundedSender<(Id, OwnedMessage)>,
    event_sink: UnboundedSender<Event>,
    users: HashMap<Id, User>,
    rng: rand::ThreadRng,
}

#[derive(Serialize, Clone)]
struct GameStateInfo {
    your_id: Id,
    users: Vec<User>,
}

#[derive(Serialize)] 
enum OutputMessage {
    GameStateInfo(GameStateInfo),
    Fire(Id, Fire),
}

impl Runner {
    pub fn new(
        handle: Handle,
        output_sink: UnboundedSender<(Id, OwnedMessage)>,
        event_sink: UnboundedSender<Event>
    ) -> Self {
        Self {
            game_state: GameState::new(),
            handle: handle,
            output_sink: output_sink,
            event_sink: event_sink,
            users: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }

    pub fn proc_event(&mut self, event: Event) {
        match event {
            Event::UserMessage(id, user_msg) => self.proc_user_event(id, user_msg),
            Event::Connect(user) => self.user_connect(user),
            Event::Disconnect(user) => self.user_disconnect(user),
        }
    }

    #[allow(unused_variables)]
    fn user_connect(&mut self, id: Id) {
    }

    #[allow(unused_variables)]
    fn user_disconnect(&mut self, id: Id) {
        info!(logger, "User {} disconnected", id);
        self.users.remove(&id);
    }

    fn proc_user_event(&mut self, id: Id, user_msg: UserMessage) {
        match user_msg {
            UserMessage::Join => {
                self.new_user(id);
            }
            UserMessage::Fire(data) => {
                self.user_fire(id, data); 
            }
        }
    }

    fn new_user(&mut self, id: Id) {
        info!(logger, "User {} joined", id);
        if self.users.contains_key(&id) {
            return;
        }

        let new_user = User {
            id: id,
            pos: Point {
                x: (self.rng.next_u32() % (GAME_WIDTH - 100) + 50) as f64,
                y: (self.rng.next_u32() % (GAME_HEIGHT - 100) + 50) as f64,
            },
            health: Health {
                max: 100.,
                value: 100.,
            }
        };

        self.users.insert(id, new_user);

        for id in self.users.keys() {
            self.send_game_info(*id);
        }
    }

    fn user_fire(&self, id: Id, data: Fire) {
        for user in self.users.keys() {
            self.send_msg(*user, OutputMessage::Fire(id, data.clone()));
        }
    }

    fn send_game_info(&self, id: Id) {
        self.send_msg(id, OutputMessage::GameStateInfo(GameStateInfo {
            your_id: id,
            users: self.users.values().map(|x| x.clone()).collect(),
        }));
    }

    fn send_msg(&self, id: Id, msg: OutputMessage) {
        self.send(id, serde_json::to_string(&msg).unwrap());
    }

    fn send(&self, id: Id, msg: String) {
        self.handle.spawn(consume_result!(
            self.output_sink.clone().send((id, OwnedMessage::Text(msg)))
        ));
    }
}
