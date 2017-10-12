use common::*;
use super::data_struct::*;
use super::state::GameState;
use super::problem::ProblemOut;

#[derive(Serialize)] 
pub enum Output<'a> {
    Initial(Initial),
    SyncGameState(&'a GameState),
    UserAdd(User),
    UserRemote(Id),
    Fire(FireOut),
    Problem(ProblemOut),
}
