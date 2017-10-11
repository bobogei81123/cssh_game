use common::*;
use super::data_struct::*;
use super::state::GameState;

#[derive(Serialize)] 
pub enum Output<'a> {
    Initial(Initial),
    SyncGameState(&'a GameState),
    UserAdd(User),
    UserRemote(Id),
    Fire(Id, Fire),
}
