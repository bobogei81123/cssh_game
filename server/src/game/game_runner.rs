use common::*;
use std::rc::Rc;
use std::cell::RefCell;
use rand::{thread_rng, ThreadRng};

use super::common::*;
use super::room;

#[derive(Serialize)]
pub struct Health {
    pub max: f64,
    pub value: f64,
}

impl Default for Health {
    fn default() -> Health {
        Health {
            max: 100.,
            value: 100.,
        }
    }
}

#[derive(Serialize)]
pub struct Player {
    id: Id, 
    name: String,
    team: usize,
    entered: bool,
    pos: Point,
    health: Health,
}

impl Player {
    pub fn new(_player: room::Player, pos: Point) -> Self {
        Self {
            id: _player.id,
            name: _player.name,
            team: _player.team,
            entered: false,
            pos: Point::new(0., 0.),
            health: Health::default(),
        }
    }
}

pub struct Game {
    common: Common,
    sink_map: Rc<RefCell<SinkMap>>,
    rng: ThreadRng,
}

impl Game {
    pub fn new<T>(
        common: Common,
        sink_map: Rc<RefCell<SinkMap>>,
        players: T) -> Self 
        where T: IntoIterator<Item=room::Player>
    {
        let pts = vec![];
        let mut rng = thread_rng(); 
        let players = {
            let ref_rng = &mut rng;
            players.into_iter().map(move |p| {
                let new_point = utils::generate_random_point(
                    ref_rng,
                    (PLAYER_AREA_X_MARGIN/2., GAME_WIDTH - PLAYER_AREA_X_MARGIN/2.),
                    (PLAYER_AREA_Y_MARGIN/2., GAME_HEIGHT - PLAYER_AREA_Y_MARGIN/2.),
                    USER_MIN_DISTANCE,
                    &pts,
                );
                (p.id, Player::new(p, new_point))
            }).collect::<HashMap<_, _>>()
        };
        Self {
            common: common,
            sink_map: sink_map,
            rng: rng,
        }
    }
}

impl_loggable!(Game);

//#[derive(Serialize)]
//pub enum Output<'a> {
//}

//impl_output_sender_lifetime!(Game, Output<'a>);

#[derive(Deserialize)]
pub enum Message {
}

impl MessageSink for Game {
    type Message = Message;

    fn proc_message(&mut self, id: Id, msg: Message) {
        match msg {
        }
    }
}

impl UserEventListener for Game {
}

impl RawService for Game {
}

