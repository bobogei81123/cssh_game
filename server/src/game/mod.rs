use std::collections::HashMap;
use std::time::Duration;
use futures::sync::mpsc::UnboundedSender;
use tokio_core::reactor::{Handle, Timeout};
use futures::{Future, Sink};
use rand::{self, Rng};
use std::boxed::FnBox;
use serde_json;
use itertools::Itertools;
use std::mem;

use websocket::message::OwnedMessage;

mod point;
mod data_struct;
mod user_send;
mod state;
mod output;
mod constant;
mod problem;

use common::*;
use event::Event;
use self::point::Point;
use self::data_struct::*;
pub use self::user_send::UserSend;
use self::state::*;
use self::output::Output;
use self::constant::*;
use self::problem::{Problem, parse_file, ProblemOut};

pub struct Runner {
    data: GameData,
    handle: Handle,
    output_sink: UnboundedSender<(Id, OwnedMessage)>,
    event_sink: UnboundedSender<Event>,
    rng: rand::ThreadRng,
    problems: Vec<Problem>,
}

impl Runner {
    pub fn new(
        handle: Handle,
        output_sink: UnboundedSender<(Id, OwnedMessage)>,
        event_sink: UnboundedSender<Event>
    ) -> Self {
        Self {
            data: GameData::new(),
            handle: handle,
            output_sink: output_sink,
            event_sink: event_sink,
            rng: rand::thread_rng(),
            problems: vec![],
        }
    }

    pub fn init(&mut self) {
        self.problems = parse_file("problems/problem.toml");
    }

    pub fn proc_event(&mut self, event: Event) {
        match event {
            Event::UserSend(id, user_send) => self.proc_user_event(id, user_send),
            Event::Connect(user) => self.user_connect(user),
            Event::Disconnect(user) => self.user_disconnect(user),
            Event::Timeout(func) => {
                func.call_box((self, ));
            }
        }
    }

    #[allow(unused_variables)]
    fn user_connect(&mut self, id: Id) {
    }

    #[allow(unused_variables)]
    fn user_disconnect(&mut self, id: Id) {
        info!(logger, "User {} disconnected", id);

        if self.data.game_state == GameState::Preparing {
            match self.data.users.remove(&id) {
                Some(user) => {
                    let team = &mut self.data.teams[user.team];
                    team.remove_item(&id);
                }
                None => (),
            }
        }
        //self.state.remove_user(id);
        //for id in self.state.users.keys() {
            //self.send(*id, &Output::SyncGameState(&self.state));
        //}
    }

    fn proc_user_event(&mut self, id: Id, user_send: UserSend) {
        match user_send {
            UserSend::RequestInitial => {
                self.user_initial(id);
            }
            UserSend::Join(name) => {
                self.user_join_room(id, name);
            }
            UserSend::Ready => {
                self.user_ready(id);
            }
            UserSend::RequestPlayersData => {
                self.send(id, &Output::PlayersData(self.data.get_player_data()));
            }
            UserSend::RequestProblem => {
                self.assign_problem(id);
            }
            UserSend::Answer(answer) => {
                self.answer(id, answer);
            }
            UserSend::Fire(data) => {
                self.user_fire(id, data); 
            }
        };
    }

    fn user_initial(&mut self, id: Id) {
        self.data.users.insert(id, User::new(id));
        self.send(id, &Output::Initial(Initial {
            id: id,
        }));
    }

    fn user_join_room(&mut self, id: Id, name: String) {
        if self.data.game_state != GameState::Preparing { return; }

        {
            let teams = &mut self.data.teams;
            let team;
            if teams[0].len() <= teams[1].len() {
                team = 0;
            } else {
                team = 1;
            }
            teams[team].push(id);
            let user = self.data.users.get_mut(&id).unwrap();
            user.team = team;
            user.name = name;
        }

        for _id in self.data.users.keys() {
            self.send(*_id, &Output::RoomData(self.data.get_room_data()));
        }
    }

    fn user_ready(&mut self, id: Id) {
        self.data.users.get_mut(&id).unwrap().ready = true;

        if self.data.users.values().all(|x| x.ready) {
            self.all_user_ready();
        }
    }

    fn all_user_ready(&mut self) {
        self.game_start();
        for id in self.data.players.keys() {
            self.send(*id, &Output::GameStart);
        }
    }

    fn game_start(&mut self) {
        info!(logger, "Game start!");
        self.data.game_state = GameState::Started;

        let mut users = HashMap::new();
        mem::swap(&mut users, &mut self.data.users);

        self.data.players = users.into_iter().map({
            let mut pts = vec![];
            let rng = &mut self.rng;

            move |(id, user)| {
                loop {
                    let pos = Point {
                        x: ((rng.next_u32() as f64) % (GAME_WIDTH - 100.) + 50.) as f64,
                        y: ((rng.next_u32() as f64) % (GAME_HEIGHT - 100.) + 50.) as f64,
                    };

                    if pts.iter().all(|p: &Point| (*p - pos).abs() >= 10.0) { 
                        pts.push(pos);
                        break; 
                    }
                }

                (id, Player {
                    id: user.id,
                    name: user.name,
                    team: user.team,
                    pos: Point {
                        x: ((rng.next_u32() as f64) % (GAME_WIDTH - 100.) + 50.) as f64,
                        y: ((rng.next_u32() as f64) % (GAME_HEIGHT - 100.) + 50.) as f64,
                    },
                    health: Health {
                        max: 100.,
                        value: 100.,
                    },
                    assigned_problem: None,
                    state: UserState::Waiting,
                })
            }
        }).collect();
    }

    fn assign_problem(&mut self, id: Id) {
        let prob = {
            let user = self.data.players.get_mut(&id).unwrap();

            let id = match user.assigned_problem {
                Some(id) => id,
                None => {
                    let n = self.problems.len();
                    self.rng.gen_range(0, n)
                }
            };

            user.assigned_problem = Some(id);

            &self.problems[id]
        };

        let output = Output::Problem(ProblemOut {
            question: prob.question.clone(),
            answers: {
                let mut vec = prob.answers.clone();
                //self.rng.shuffle(&mut vec);
                vec
            }
        });

        self.send(id, &output);
    }

    fn answer(&mut self, id: Id, answer: usize) {
        let problem_id = self.data.players.get(&id).unwrap().assigned_problem;
        let problem_id = match problem_id {
            None => { return; }
            Some(pid) => pid
        };
        let result = answer == self.problems[problem_id].correct;
        self.send(id, &Output::JudgeResult(result));
        
        {
            let user = self.data.players.get_mut(&id).unwrap();

            if result {
                user.state = UserState::Firing;
            } else {
                user.state = UserState::Penalizing;
            }

            user.assigned_problem = None;
        }

        //if (!result) {
            //self.exec_timeout(Box::new(move |s: &mut Runner| {
                //s.get_mut_user(id).unwrap().state = UserState::Waiting;
            //}), Duration::from_secs(2));
        //}
    }

    //fn exec_timeout<F>(&self, f: Box<F>, duration: Duration)
        //where for<'r> F: FnBox(&'r mut Runner) -> () + Send + 'static {
        //let future = Timeout::new(duration, &self.handle).unwrap()
            //.and_then({
                //let event_sink = self.event_sink.clone();
                //let handle = self.handle.clone();
                //move |_| {
                    //handle.spawn(consume_result!(event_sink.send(Event::Timeout(f))));
                    //Ok(())
                //}
            //}).map_err(|_| ());
        //self.handle.spawn(future);
    //}

    //fn new_user(&mut self, id: Id) {
        //info!(logger, "User {} joined", id);
        //if self.state.users.contains_key(&id) {
            //return;
        //}

        //let new_user = User {
            //id: id,
            //pos: Point {
                //x: ((self.rng.next_u32() as f64) % (GAME_WIDTH - 100.) + 50.) as f64,
                //y: ((self.rng.next_u32() as f64) % (GAME_HEIGHT - 100.) + 50.) as f64,
            //},
            //health: Health {
                //max: 100.,
                //value: 100.,
            //},
            //assigned_problem: None,
            //state: UserState::Waiting,
        //};

        //self.state.add_user(new_user);

        //self.send(id, &Output::Initial(Initial { your_id: id }));

        //for id in self.state.users.keys() {
            //self.send(*id, &Output::SyncGameState(&self.state));
        //}
    //}

    fn _get_distant_to_line(o: Point, angle: f64, x: Point) -> (f64, f64) {
        let unit = Point::from_angle(angle);
        let d = x - o;
        let dp = d * unit;
        let dd = f64::sqrt(d * d - dp * dp);

        (dp, dd)
    }

    fn user_fire(&mut self, id: Id, data: Fire) {

        let my_pos = self.data.players.get(&id).unwrap().pos;

        let result = self.data.players.values().fold(
            None,
            |x, player| {
                let target = player.id;
                if target == id { return x; }
                let (dis_par, dis_oth) = Self::_get_distant_to_line(
                    my_pos, data.angle, player.pos
                );

                if dis_par < 0. || dis_oth > USER_RADIUS { return x; }

                match x {
                    None => Some((target, dis_par, dis_oth)),
                    Some(best) => {
                        let (_, best_par, _) = best;
                        if best_par > dis_par { Some((target, dis_par, dis_oth)) }
                        else { Some(best) }
                    }
                }
            }
        );

        //let damage_val = result.map(|(_, _, d)
            //40 * (USER_RADIUS - d) / USER_RADIUS + 10; 
        //)

        let damage_val = result.map(|(target, dis_par, dis_oth)| (
            target,
            35. * (USER_RADIUS - dis_oth) / USER_RADIUS + 15., 
        ));



        let damage = match damage_val {
            Some((target, val)) => {
                let health_after = self.data.damage(target, val);
                Some(Damage {
                    target: target,
                    value: val,
                    health_after: health_after,
                })
            }
            None => None,
        };


        let output = Output::Fire(FireOut {
            id: id,
            fire: data,
            damage: damage,
        });

        for id in self.data.players.keys() {
            self.send(*id, &output);
        }
    }

    fn send(&self, id: Id, msg: &Output) {
        self._send(id, serde_json::to_string(&msg).unwrap());
    }

    fn _send(&self, id: Id, msg: String) {
        self.handle.spawn(consume_result!(
            self.output_sink.clone().send((id, OwnedMessage::Text(msg)))
        ));
    }
}
