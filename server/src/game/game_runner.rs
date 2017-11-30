use common::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::boxed::FnBox;
use std::time::Duration;
use rand::{thread_rng, ThreadRng, Rng};
use tokio_core::reactor::Timeout;

use super::common::*;
use super::room;
use super::problem::{Problem, parse_file};

use futures::unsync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};


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

impl Health {
    #[allow(dead_code)]
    pub fn add(&mut self, v: f64) {
        self.value = f64::min(self.max, self.value + v);
    }

    pub fn sub(&mut self, v: f64) {
        self.value = f64::max(0f64, self.value - v);
    }
}

enum PlayerState {
    Waiting,
    Answering(usize),
    Firing,
}

#[derive(Serialize)]
pub struct Player {
    id: Id, 
    name: String,
    team: usize,
    pos: Point,
    health: Health,

    #[serde(skip)]
    entered: bool,
    #[serde(skip)]
    state: PlayerState,
}

impl Player {
    pub fn new(_player: room::Player, pos: Point) -> Self {
        Self {
            id: _player.id,
            name: _player.name,
            team: _player.team,
            pos: pos,
            health: Health::default(),
            entered: false,
            state: PlayerState::Waiting,
        }
    }
}

pub struct TimeoutEvent(Box<for <'r> FnBox(&'r mut Game) -> ()>);

pub struct Game {
    common: Common,
    sink_map: Rc<RefCell<SinkMap>>,
    rng: ThreadRng,
    players: HashMap<Id, Player>,
    started: bool,
    timeout_event_sink: UnboundedSender<TimeoutEvent>,
    problems: Vec<Problem>,
}

impl Game {
    pub fn new<T>(
        common: Common,
        sink_map: Rc<RefCell<SinkMap>>,
        players: T,
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
            started: false,
            timeout_event_sink: timeout_event_sink,
        }));
        
        handle.spawn(timeout_event_stream.for_each({
            let mut me = me.clone();
            move |TimeoutEvent(f)| {
                f.call_box((&mut *me.borrow_mut(),));
                Ok(())
            }
        }));

        me
    }
}

impl_loggable!(Game);

#[derive(Serialize, Debug)]
pub struct Damage {
    pub target: Id,
    pub value: f64,
    pub health_after: f64,
}

#[derive(Serialize, Debug)]
pub struct FireResult {
    pub id: Id,
    pub fire: Fire,
    pub damage: Option<Damage>,
}

#[derive(Serialize)]
pub enum Output<'a> {
    PlayersData(&'a HashMap<Id, Player>),
    Problem(&'a Problem),
    StartFire,
    FireResult(FireResult),
    JudgeResult(bool),
}

impl_output_sender_lifetime!(Game, Output<'a>);

#[derive(Serialize, Deserialize, Debug)]
pub struct Fire {
    pub pos: Point,
    pub angle: f64,
}

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

    fn run_after<F>(&self, f: Box<F>, duration: Duration)
        where for<'r> F: (FnBox(&'r mut Self) -> ()) + 'static {

        let future = Timeout::new(duration, &self.common.handle).unwrap()
            .and_then({
                let timeout_event_sink = self.timeout_event_sink.clone();
                move |_| {
                    timeout_event_sink.unbounded_send(TimeoutEvent(f));
                    Ok(())
                }
            }).map_err(|_| ());

        self.common.handle.spawn(future);
    }

    fn assign_problem(&mut self, id: Id) {
        do catch {
            let prob = {
                let player = self.players.get_mut(&id)?;
                if let PlayerState::Waiting = player.state {} else { None? }

                let n = self.problems.len();
                let prob_id = self.rng.gen_range(0, n);
                player.state = PlayerState::Answering(prob_id);

                &self.problems[prob_id]
            };

            self.send(id, &Output::Problem(prob));
            Some(())
        };
    }

    fn answer(&mut self, id: Id, answer: usize) {
        do catch {
            let result = {
                let player = self.players.get(&id)?;
                let prob_id = {
                    if let PlayerState::Answering(prob_id) = player.state { prob_id } else { None? }
                };
                let result = answer == self.problems[prob_id].correct;
                self.send(id, &Output::JudgeResult(result));
                result
            };

            {
                let player = self.players.get_mut(&id).unwrap();
                if result {
                    player.state = PlayerState::Firing;
                } else {
                    player.state = PlayerState::Waiting;
                }
            }

            if result {
                self.send(id, &Output::StartFire);
            } else {
                self.run_after(box move |me: &mut Game| {
                    me.assign_problem(id);
                }, Duration::from_secs(3));
            }

            Some(())
        };
    }

    fn _get_distant_to_line(o: Point, angle: f64, x: Point) -> (f64, f64) {
        let unit = Point::from_angle(angle);
        let d = x - o;
        let dp = d * unit;
        let dd = f64::sqrt(d * d - dp * dp);

        (dp, dd)
    }

    fn player_fire(&mut self, id: Id, data: Fire) {

        let result = {
            let player = match self.players.get(&id) {
                None => { return; }
                Some(player) => player,
            };

            self.players.values().fold(
                None,
                |x, ref other| {
                    if other.team == player.team { return x; }
                    let (dis_par, dis_oth) = Self::_get_distant_to_line(
                        player.pos, data.angle, other.pos);

                    if dis_par < 0. || dis_oth > USER_RADIUS { return x; }

                    match x {
                        None => Some((other.id, dis_par, dis_oth)),
                        Some(best) => {
                            let (_, best_par, _) = best;
                            if best_par > dis_par { Some((other.id, dis_par, dis_oth)) }
                            else { Some(best) }
                        }
                    }
                }
            )
        };

        let damage_val = result.map(|(target, dis_par, dis_oth)| (
            target,
            40. * (USER_RADIUS - dis_oth) / USER_RADIUS + 10., 
        ));


        let damage = match damage_val {
            Some((target, val)) => {
                let target = self.players.get_mut(&target).unwrap();
                target.health.sub(val);
                let health_after = target.health.value;

                Some(Damage {
                    target: target.id,
                    value: val,
                    health_after: health_after,
                })
            }
            None => None,
        };


        let output = Output::FireResult(FireResult {
            id: id,
            fire: data,
            damage: damage,
        });

        self.send_many(self.players.keys(), &output);
    }
}
