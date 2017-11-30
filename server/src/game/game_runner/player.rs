use super::*;
use game::room;

pub enum PlayerState {
    Waiting,
    Answering(usize),
    Firing,
}

#[derive(Serialize)]
pub struct Player {
    pub id: Id, 
    pub name: String,
    pub team: usize,
    pub pos: Point,
    pub health: Health,
    pub alive: bool,

    #[serde(skip)]
    pub entered: bool,
    #[serde(skip)]
    pub state: PlayerState,
}

impl Player {
    pub fn new(_player: room::Player, pos: Point) -> Self {
        Self {
            id: _player.id,
            name: _player.name,
            team: _player.team,
            pos: pos,
            health: Health::default(),
            alive: true,
            entered: false,
            state: PlayerState::Waiting,
        }
    }
}
