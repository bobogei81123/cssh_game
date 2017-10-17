use std::collections::HashMap;
use common::*;
use super::data_struct::*;

pub struct GameData {
    pub game_state: GameState,
    pub users: HashMap<Id, User>,
    pub teams: [Vec<Id>; 2],
    pub players: HashMap<Id, Player>,
    pub spectators: Vec<Id>,
    //pub users_game_data:
}

#[derive(PartialEq)]
pub enum GameState {
    Preparing,
    Started,
}

#[derive(Debug, Clone)]
pub enum UserState {
    Waiting,
    Answering,
    Firing,
    Penalizing,
}

#[derive(Serialize)]
pub struct RoomData<'a> {
    pub users: &'a HashMap<Id, User>,
    pub teams: &'a [Vec<Id>; 2],
}

#[derive(Serialize)]
pub struct PlayersData<'a> {
    pub teams: &'a [Vec<Id>; 2],
    pub players: &'a HashMap<Id, Player>,
}

impl GameData {
    pub fn new() -> Self {
        Self { 
            game_state: GameState::Preparing,
            users: HashMap::new(),
            teams: [vec![], vec![]],
            players: HashMap::new(),
            spectators: vec![],
        }
    }

    pub fn get_room_data(&self) -> RoomData {
        RoomData {
            users: &self.users,
            teams: &self.teams,
        }
    }

    pub fn get_player_data(&self) -> PlayersData {
        PlayersData {
            teams: &self.teams,
            players: &self.players,
        }
    }


    //pub fn add_user(&mut self, user: User) {
        //self.users.insert(user.id, user);
    //}

    //pub fn remove_user(&mut self, id: Id) {
        //self.users.remove(&id);
    //}

    pub fn damage(&mut self, id: Id, val: f64) -> f64 {
        let who = self.players.get_mut(&id).unwrap();
        who.health.sub(val);
        who.health.value
    }
}
