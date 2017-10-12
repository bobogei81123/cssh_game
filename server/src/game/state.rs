use std::collections::HashMap;
use common::*;
use super::data_struct::User;

#[derive(Serialize)]
pub struct GameState {
    pub users: HashMap<Id, User>,
}

#[derive(Debug, Clone)]
pub enum UserState {
    Waiting,
    Answering,
    Firing,
    Penalizing,
}


impl GameState {
    pub fn new() -> Self {
        Self { 
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    pub fn remove_user(&mut self, id: Id) {
        self.users.remove(&id);
    }

    pub fn damage(&mut self, id: Id, val: f64) -> f64 {
        let who = self.users.get_mut(&id).unwrap();
        who.health.sub(val);
        who.health.value
    }
}
