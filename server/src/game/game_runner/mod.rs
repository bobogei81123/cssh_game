mod health;
mod player;
mod problem;
mod fire;
mod constant;

extern crate boolinator;

use common::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::boxed::FnBox;
use std::time::Duration;
use rand::{thread_rng, ThreadRng, Rng};
use tokio_core::reactor::Timeout;

use self::boolinator::Boolinator;

use super::common::*;
use super::room;

use futures::unsync::mpsc::{unbounded, UnboundedSender};

use self::constant::*;
use self::health::Health;
use self::player::{Player, PlayerState};
use self::problem::{Problem, parse_file};
use self::fire::*;

pub struct TimeoutEvent(Box<for <'r> FnBox(&'r mut Game) -> ()>);

pub struct Game {
    common: Common,
    sink_map: Rc<RefCell<SinkMap>>,
    rng: ThreadRng,
    players: HashMap<Id, Player>,
    teams: [Vec<Id>; 2],
    started: bool,
    timeout_event_sink: UnboundedSender<TimeoutEvent>,
    problems: Vec<Problem>,
}

impl Game {
    pub fn new<T>(
        common: Common,
        sink_map: Rc<RefCell<SinkMap>>,
        players: T,
        teams: [Vec<Id>; 2],
    ) -> Rc<RefCell<Self>>
        where T: IntoIterator<Item=room::Player>
    {

        let handle = common.handle.clone();
        let mut pts = vec![];
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
                pts.push(new_point);
                (p.id, Player::new(p, new_point))
            }).collect::<HashMap<_, _>>()
        };
        let (timeout_event_sink, timeout_event_stream) = unbounded();
        let me = Rc::new(RefCell::new(Self {
            common: common,
            sink_map: sink_map,
            problems: parse_file("problems/problem.toml"),
            rng: rng,
            players: players,
            teams: teams,
            started: false,
            timeout_event_sink: timeout_event_sink,
        }));
        
        handle.spawn(timeout_event_stream.for_each({
            let me = me.clone();
            move |TimeoutEvent(f)| {
                f.call_box((&mut *me.borrow_mut(),));
                Ok(())
            }
        }));

        me
    }
}

impl_loggable!(Game);

#[derive(Serialize)]
pub enum Output<'a> {
    PlayersData(&'a HashMap<Id, Player>),
    Problem(&'a Problem),
    StartFire,
    FireResult(FireResult),
    Damage(Damage),
    JudgeResult(bool),
    TeamWin(usize),
}

impl_output_sender_lifetime!(Game, Output<'a>);

#[derive(Deserialize)]
pub enum Message {
    Entered,
    Answer(usize),
    Fire(Fire),
}

impl MessageSink for Game {
    type Message = Message;

    fn proc_message(&mut self, id: Id, msg: Message) {
        match msg {
            Message::Entered => {
                self.player_enter(id);
            }
            Message::Answer(answer) => {
                self.answer(id, answer);
            }
            Message::Fire(data) => {
                self.player_fire(id, data);
            }
        }
    }
}


impl UserEventListener for Game {
    fn user_disconnect(&mut self, id: Id) {
        self.user_dead(id);
    }
}

impl RawService for Game {
}

impl Game {
    fn player_enter(&mut self, id: Id) {
        do catch {
            {
                let player = self.players.get_mut(&id)?;
                player.entered = true;
            }
            if self.players.values().all(|x| x.entered) {
                self.start_game();
            }
            Some(())
        };
    }

    fn start_game(&mut self) {
        self.started = true;

        let output = &Output::PlayersData(
            &self.players
        );
        self.send_many(self.players.keys(), output);

        self.run_after(box |me: &mut Self| {
            for id in me.players.keys().cloned().collect::<Vec<_>>() {
                me.assign_problem(id);
            }
        }, Duration::from_secs(1))
    }

    fn user_dead(&mut self, id: Id) -> Option<()> {
        let team = {
            let player = self.players.get_mut(&id)?;
            player.alive = false;
            player.team
        };

        if self.players
            .values()
            .filter(|other| other.team == team)
            .all(|other| !other.alive) {
            self.team_win(if team == 0 { 1 } else { 0 });
        }
        Some(())
    }

    fn team_win(&mut self, team: usize) { 
        info!(self.logger(), "Team #{} won!", team);
        self.send_many(
            self.players.keys(),
            &Output::TeamWin(team),
        );
    }

    #[allow(boxed_local)]
    fn run_after<F: Sized>(&self, f: Box<F>, duration: Duration)
        where for<'r> F: (FnBox(&'r mut Self) -> ()) + 'static {

        let future = Timeout::new(duration, &self.common.handle).unwrap()
            .and_then({
                let timeout_event_sink = self.timeout_event_sink.clone();
                move |_| {
                    timeout_event_sink.unbounded_send(TimeoutEvent(f)).unwrap();
                    Ok(())
                }
            }).map_err(|_| ());

        self.common.handle.spawn(future);
    }

}
