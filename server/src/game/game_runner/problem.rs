use std::fs::File;
use std::io::Read;
use rand::{self, Rng};
use toml;
use super::*;

#[derive(Deserialize)]
struct _Problem {
    question: String,
    answers: Vec<String>,
}

#[derive(Deserialize)]
struct _Problems {
    problems: Vec<_Problem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Problem {
    pub question: String,
    pub answers: Vec<String>,
    #[serde(skip_serializing)]
    pub correct: usize,
}

pub fn parse_file(path: &str) -> Vec<Problem> {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let problems: _Problems = toml::from_str(&contents).unwrap();

    let mut rng = rand::thread_rng();

    problems
        .problems
        .into_iter()
        .map(|prob| {
            let n = prob.answers.len();
            let mut ord: Vec<_> = (0..n).collect();
            rng.shuffle(ord.as_mut_slice());
            let shuffled = ord.iter().map(|i| prob.answers[*i].clone()).collect();
            let correct = ord.iter().position(|&x| x == 0).unwrap();
            Problem {
                question: prob.question,
                answers: shuffled,
                correct: correct,
            }
        })
        .collect()
}

impl Game {
    pub fn assign_problem(&mut self, id: Id) -> Option<()> {
        let prob = {
            let player = self.players.get_mut(&id)?;
            if let PlayerState::Waiting = player.state {} else { 
                return None;
            }

            let n = self.problems.len();
            let prob_id = self.rng.gen_range(0, n);
            player.state = PlayerState::Answering(prob_id);

            &self.problems[prob_id]
        };

        self.send(id, &Output::Problem(prob));
        Some(())
    }

    pub fn answer(&mut self, id: Id, answer: usize) -> Option<()> {
        let result = {
            let player = self.players.get(&id)?;
            let prob_id = {
                if let PlayerState::Answering(prob_id) = player.state {
                    prob_id
                } else { return None; }
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
    }
}
