use common::*;

use std::mem;

use super::common::*;
use super::lobby::User;
use super::game_runner::Game;

#[derive(Serialize)]
pub struct Player {
    pub id: Id,
    pub name: String,
    pub team: usize,
    ready: bool,
}

impl Player {
    fn new(User(id, name): User, team: usize) -> Self {
        Self {
            id: id,
            name: name,
            ready: false,
            team: team,
        }
    }
}

pub struct Room {
    common: Common,
    sink_map: Rc<RefCell<SinkMap>>,
    players: HashMap<Id, Player>,
    teams: [Vec<Id>; 2],
}

impl_loggable!(Room);

#[derive(Serialize)]
pub struct RoomData<'a> {
    players: &'a HashMap<Id, Player>,
    teams: &'a [Vec<Id>; 2],
}

#[derive(Serialize)]
pub enum Output<'a> {
    RoomData(RoomData<'a>),
    GameStart,
}

impl_output_sender_lifetime!(Room, Output<'a>);

#[derive(Deserialize)]
pub enum Message {
    Entered,
    Ready,
}

impl MessageSink for Room {
    type Message = Message;

    fn proc_message(&mut self, id: Id, msg: Message) {
        match msg {
            Message::Entered => {
                self.send(id, &self.room_data());
            }
            Message::Ready => {
                self.user_ready(id);
            }
        }
    }
}

impl RawService for Room {}

impl UserEventListener for Room {
    fn user_disconnect(&mut self, id: Id) {
        do catch {
            let player = self.players.remove(&id)?;
            let team = player.team;
            self.teams[team].remove_item(&id).unwrap();
            self.broadcast_data();
            Some(())
        };
    }
}


impl Room {
    fn room_data(&self) -> Output {
        Output::RoomData(RoomData {
            players: &self.players,
            teams: &self.teams,
        })
    }

    fn broadcast_data(&self) {
        self.send_many(self.players.keys(), &self.room_data());
    }
}

impl Room {
    pub fn new(common: Common, sink_map: Rc<RefCell<SinkMap>>) -> Self {
        Self {
            common: common,
            sink_map: sink_map,
            players: HashMap::new(),
            teams: [vec![], vec![]],
        }
    }

    pub fn user_entering(&mut self, user: User) {
        let id = user.0;
        info!(self.logger(), "User enter room"; "id" => user.0, "name" => &user.1);
        let team = if self.teams[0].len() <= self.teams[1].len() { 0 } else { 1 };
        self.players.insert(id, Player::new(user, team));
        self.teams[team].push(id);
        self.broadcast_data();
    }

    fn user_ready(&mut self, id: Id) {
        do catch {
            {
                let player = self.players.get_mut(&id)?;
                player.ready = true;
            }
            self.broadcast_data();

            if self.players.values().all(|x| x.ready) {
                self.spawn_game();
            }
            Some(())
        };
    }

    fn spawn_game(&mut self) {
        self.send_many(self.players.keys(), &Output::GameStart);

        let players = mem::replace(&mut self.players, HashMap::new());

        let ids = players.keys().cloned().collect::<Vec<_>>();

        let game = Game::new(Common {
            logger: self.logger().new(o!("who" => "Game")),
                ..self.common.clone()
            }, self.sink_map.clone(), players.into_iter().map(|(_id, p)| p)
        );

        let mut sink_map = self.sink_map.borrow_mut();
        for id in ids {
            sink_map.insert(id, game.clone());
        }
    }
}
