use std::fs::File;
use std::io::Read;
use rand::{self, Rng};
use toml;

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
