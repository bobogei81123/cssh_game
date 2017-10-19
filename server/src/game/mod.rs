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

macro_rules! require_game_state {
    ($s: ident, $e: expr) => {
        if $s.data.game_state != $e { return; }
    }
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

        match self.data.game_state {
            GameState::Preparing => {
                let user = ensure!(self.data.users.remove(&id));
                let team = &mut self.data.teams[user.team];
                team.remove_item(&id);
            }
            GameState::Started => {
                if self.data.players.contains_key(&id) {
                    self.user_dead(id);
                }
            }
        }
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
        require_game_state!(self, GameState::Preparing);

        self.data.users.insert(id, User::new(id));
        self.send(id, &Output::Initial(Initial {
            id: id,
        }));
    }

    fn user_join_room(&mut self, id: Id, name: String) {
        require_game_state!(self, GameState::Preparing);

        if name == "spectator" {
            let user = ensure!(self.data.users.get_mut(&id));
            self.data.spectators.push(id);
            user.team = 2;
            user.name = name;
            return;
        }

        {
            let user = ensure!(self.data.users.get_mut(&id));
            let teams = &mut self.data.teams;
            let team;
            if teams[0].len() <= teams[1].len() {
                team = 0;
            } else {
                team = 1;
            }
            teams[team].push(id);
            user.team = team;
            user.name = name;
        }

        self.send_all(
            self.data.users.keys(), 
            &Output::RoomData(self.data.get_room_data())
        );
    }

    fn user_ready(&mut self, id: Id) {
        require_game_state!(self, GameState::Preparing);

        {
            let user = ensure!(self.data.users.get_mut(&id));
            user.ready = true;
        }

        if self.data.users.values().all(|x| x.ready) {
            self.game_start();
        }
    }

    fn game_start(&mut self) {
        require_game_state!(self, GameState::Preparing);
        info!(logger, "Game start!");

        let mut users = HashMap::new();
        mem::swap(&mut users, &mut self.data.users);


        self.data.players = {
            let mut generate_point = || Point {
                x: ((self.rng.next_u32() as f64) % (GAME_WIDTH - GAME_WIDTH_MARGIN)
                    + GAME_WIDTH_MARGIN/2.) as f64,
                y: ((self.rng.next_u32() as f64) % (GAME_HEIGHT - GAME_HEIGHT_MARGIN)
                    + GAME_HEIGHT_MARGIN/2.) as f64,
            };

            users.into_iter().filter(|u| u.1.team != 2).map({
                let mut pts = vec![];

                move |(id, user)| {
                    let pos = loop {
                        let pos = generate_point();
                        if pts.iter().all(|p: &Point| (*p - pos).abs() >= 100.0) { 
                            pts.push(pos);
                            break pos; 
                        }
                    };

                    (id, Player {
                        id: user.id,
                        name: user.name,
                        team: user.team,
                        pos: pos,
                        health: Health {
                            max: 100.,
                            value: 100.,
                        },
                        assigned_problem: None,
                        state: UserState::Waiting,
                        alive: true,
                    })
                }
            }).collect()
        };

        self.data.game_state = GameState::Started;
        self.send_all(self.data.players.keys().chain(self.data.spectators.iter()), &Output::GameStart);
    }

    fn assign_problem(&mut self, id: Id) {
        require_game_state!(self, GameState::Started);
        let prob = {
            let user = ensure!(self.data.players.get_mut(&id));

            let pid = match user.assigned_problem {
                Some(pid) => pid,
                None => {
                    let n = self.problems.len();
                    self.rng.gen_range(0, n)
                }
            };

            user.assigned_problem = Some(pid);

            &self.problems[pid]
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
        require_game_state!(self, GameState::Started);

        let result = {
            let user = ensure!(self.data.players.get_mut(&id));
            let problem_id = ensure!(user.assigned_problem);
            user.assigned_problem = None;
            answer == self.problems[problem_id].correct
        };
        self.send(id, &Output::JudgeResult(result));
        //if (!result) {
            //self.exec_timeout(Box::new(move |s: &mut Runner| {
                //s.get_mut_user(id).unwrap().state = UserState::Waiting;
            //}), Duration::from_secs(2));
        //}
    }

    fn user_dead(&mut self, id: Id) {
        require_game_state!(self, GameState::Started);

        self.data.players.get_mut(&id).unwrap().alive = false;
        for jd in self.data.players.keys() {
            self.send(*jd, &Output::Dead(id));
        }
        self.check_win();
    }

    fn check_win(&mut self) {
        require_game_state!(self, GameState::Started);

        for i in (0..2) {
            if self.data.teams[i].iter().all(|p| !self.data.players.get(&p).unwrap().alive) {
                self.team_win(1 - i);
                return;
            }
        }
    }

    fn team_win(&mut self, team: usize) {
        require_game_state!(self, GameState::Started);

        info!(logger, "Team #{} won!", team);
        self.send_all(self.data.players.keys().chain(self.data.spectators.iter()), &Output::TeamWin(team));
        self.finalize();
    }

    fn finalize(&mut self) {
        self.data = GameData::new();
    }

    fn exec_timeout<F>(&self, f: Box<F>, duration: Duration)
        where for<'r> F: FnBox(&'r mut Runner) -> () + Send + 'static {
        let future = Timeout::new(duration, &self.handle).unwrap()
            .and_then({
                let event_sink = self.event_sink.clone();
                let handle = self.handle.clone();
                move |_| {
                    handle.spawn(consume_result!(event_sink.send(Event::Timeout(f))));
                    Ok(())
                }
            }).map_err(|_| ());
        self.handle.spawn(future);
    }

    fn _get_distant_to_line(o: Point, angle: f64, x: Point) -> (f64, f64) {
        let unit = Point::from_angle(angle);
        let d = x - o;
        let dp = d * unit;
        let dd = f64::sqrt(d * d - dp * dp);

        (dp, dd)
    }

    fn user_fire(&mut self, id: Id, data: Fire) {
        require_game_state!(self, GameState::Started);
        let (my_pos, my_team) = {
            let player = ensure!(self.data.players.get(&id));
            (player.pos, player.team)
        };

        let my_pos = self.data.players.get(&id).unwrap().pos;

        let result = self.data.players.values()
            .filter(|x| x.alive && x.team != my_team)
            .fold(None,
                |x, player| {
                    let (dis_par, dis_oth) = Self::_get_distant_to_line(
                        my_pos, data.angle, player.pos
                    );

                    if dis_par < 0. || dis_oth > USER_RADIUS { return x; }

                    match x {
                        None => Some((player.id, dis_par, dis_oth)),
                        Some(best) => {
                            let (_, best_par, _) = best;
                            if best_par > dis_par { Some((player.id, dis_par, dis_oth)) }
                            else { Some(best) }
                        }
                    }
                }
            );


        let damage = match result {
            Some((target, dis_par, dis_oth)) => {
                let val = 35. * (USER_RADIUS - dis_oth) / USER_RADIUS + 15.;
                let health_after = self.data.damage(target, val);

                if health_after == 0.0f64 {
                    self.exec_timeout(Box::new(move |s: &mut Runner| {
                        s.user_dead(target);
                    }), Duration::from_millis((dis_par / 300.0 * 1000.0) as u64));
                }

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



        self.send_all(self.data.players.keys().chain(self.data.spectators.iter()),
            &output);
    }

    fn send(&self, id: Id, msg: &Output) {
        self._send(id, serde_json::to_string(&msg).unwrap());
    }

    fn _send(&self, id: Id, msg: String) {
        self.handle.spawn(consume_result!(
            self.output_sink.clone().send((id, OwnedMessage::Text(msg)))
        ));
    }

    fn send_all<'a, 'b, T>(&self, iter: T, msg: &'b Output)
        where T: Iterator<Item=&'a Id> {
        for id in iter {
            self.send(*id, msg);
        }
    }
}
