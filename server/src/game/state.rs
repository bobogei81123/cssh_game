use std::collections::HashMap;
use common::*;
use super::data_struct::User;

#[derive(Serialize)]
pub struct GameState {
    pub users: HashMap<Id, User>,
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
}
