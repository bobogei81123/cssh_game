use common::*;
use super::data_struct::*;
use super::state::*;
use super::problem::ProblemOut;

#[derive(Serialize)] 
pub enum Output<'a> {
    Initial(Initial),
    RoomData(RoomData<'a>),
    GameStart,
    PlayersData(PlayersData<'a>),
    Fire(FireOut),
    Problem(ProblemOut),
    JudgeResult(bool),
    Dead(Id),
    TeamWin(usize),
}
