use common::*;

use std::collections::HashMap;

use super::common_trait::*;
use super::lobby::User;

#[derive(Serialize)]
pub struct Player {
    id: Id,
    name: String,
    team: usize,
    ready: bool,
}

impl From<User> for Player {
    fn from(User(id, name): User) -> Self {
        Self {
            id: id,
            name: name,
            ready: false,
            team: 0,
        }
    }
}

pub struct Room {
    output_sink: WsSender,
    logger: Logger,
    players: HashMap<Id, Player>,
    teams: [Vec<Id>; 2],
}

impl Loggable for Room {
    fn get_logger(&self) -> &Logger { &self.logger }
}

#[derive(Serialize)]
pub struct RoomData<'a> {
    users: &'a HashMap<Id, Player>,
    teams: &'a [Vec<Id>; 2],
}

#[derive(Serialize)]
pub enum Output<'a> {
    RoomData(RoomData<'a>),
}

impl<'a> OutputSender for &'a Room {
    type Output = Output<'a>;

    fn get_send_sink(&self) -> &WsSender {
        &self.output_sink
    }
}

#[derive(Deserialize)]
pub enum Message {
    Entered,
}

impl Service for Room {
    type Message = Message;

    fn proc_message(&mut self, id: Id, msg: Message) {
        match msg {
            Message::Entered => {
                self.send_players_data(id);
            }
        }
    }

    fn user_disconnect(&mut self, _id: Id) {}
}

impl RawService for Room {}

impl Room {
    fn send_players_data(&self, id: Id) {
        self.send(
            id,
            &Output::RoomData(RoomData {
                users: &self.players,
                teams: &self.teams,
            })
        )
    }
}

impl Room {
    pub fn new(output_sink: WsSender, logger: Logger) -> Self {
        Self {
            output_sink: output_sink,
            logger: logger,
            players: HashMap::new(),
            teams: [vec![], vec![]],
        }
    }

    pub fn user_entering(&mut self, user: User) {
        let id = user.0;
        info!(self.logger, "User enter room"; "id" => user.0, "name" => &user.1);
        self.players.insert(id, user.into());
        self.teams[0].push(id);
    }
}

