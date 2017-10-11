use std::collections::HashMap;
use futures::sync::mpsc::UnboundedSender;
use tokio_core::reactor::Handle;
use futures::{Future, Sink};
use rand::{self, Rng};
use serde_json;

use websocket::message::OwnedMessage;

mod point;
mod data_struct;
mod user_send;
mod state;
mod output;
mod constant;

use common::*;
use event::Event;
use self::point::Point;
use self::data_struct::*;
pub use self::user_send::UserSend;
use self::state::GameState;
use self::output::Output;
use self::constant::*;


pub struct Runner {
    state: GameState,
    handle: Handle,
    output_sink: UnboundedSender<(Id, OwnedMessage)>,
    event_sink: UnboundedSender<Event>,
    rng: rand::ThreadRng,
}

impl Runner {
    pub fn new(
        handle: Handle,
        output_sink: UnboundedSender<(Id, OwnedMessage)>,
        event_sink: UnboundedSender<Event>
    ) -> Self {
        Self {
            state: GameState::new(),
            handle: handle,
            output_sink: output_sink,
            event_sink: event_sink,
            rng: rand::thread_rng(),
        }
    }

    pub fn proc_event(&mut self, event: Event) {
        match event {
            Event::UserSend(id, user_send) => self.proc_user_event(id, user_send),
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
        self.state.remove_user(id);
        for id in self.state.users.keys() {
            self.send(*id, Output::SyncGameState(&self.state));
        }
    }

    fn proc_user_event(&mut self, id: Id, user_send: UserSend) {
        match user_send {
            UserSend::Join => {
                self.new_user(id);
            }
            UserSend::Fire(data) => {
                self.user_fire(id, data); 
            }
            UserSend::RequestSyncGameState => {
                self.send(id, Output::SyncGameState(&self.state));
            }
        }
    }

    fn new_user(&mut self, id: Id) {
        info!(logger, "User {} joined", id);
        if self.state.users.contains_key(&id) {
            return;
        }

        let new_user = User {
            id: id,
            pos: Point {
                x: ((self.rng.next_u32() as f64) % (GAME_WIDTH - 100.) + 50.) as f64,
                y: ((self.rng.next_u32() as f64) % (GAME_HEIGHT - 100.) + 50.) as f64,
            },
            health: Health {
                max: 100.,
                value: 100.,
            }
        };

        self.state.add_user(new_user);

        self.send(id, Output::Initial(Initial { your_id: id }));

        for id in self.state.users.keys() {
            self.send(*id, Output::SyncGameState(&self.state));
        }
    }

    fn _get_distant_to_line(o: Point, angle: f64, x: Point) -> (f64, f64) {
        let unit = Point::from_angle(angle);
        let d = x - o;
        let dp = d * unit;
        let dd = f64::sqrt(d * d - dp * dp);

        (dp, dd)
    }

    fn user_fire(&self, id: Id, data: Fire) {

        let my_pos = self.state.users.get(&id).unwrap().pos;

        let result = self.state.users.values().fold(
            None,
            |x, ref user| {
                if user.id == id { return x; }
                let (dp, dd) = Self::_get_distant_to_line(
                    my_pos, data.angle, user.pos);

                println!("dd = {}", dd);

                if dp < 0. || dd > USER_RADIUS { return x; }

                match x {
                    None => Some((id, dp)),
                    Some(z) => {
                        let (_, y) = z;
                        if y < dp { Some((id, dp)) }
                        else { Some(z) }
                    }
                }
            }
        );

        match result { 
            None => {
                for user in self.state.users.keys() {
                    self.send(*user, Output::Fire(id, data.clone()));
                }
            }
            Some((id, dp)) => {
                info!(logger, "Hit"; "id" => id, "dp" => dp);
            }
        }
    }

    fn send(&self, id: Id, msg: Output) {
        self._send(id, serde_json::to_string(&msg).unwrap());
    }

    fn _send(&self, id: Id, msg: String) {
        self.handle.spawn(consume_result!(
            self.output_sink.clone().send((id, OwnedMessage::Text(msg)))
        ));
    }
}
