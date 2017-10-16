use super::data_struct::*;

#[derive(Deserialize, Debug)]
pub enum UserSend {
    RequestInitial,
    Join(String),
    Ready,
    RequestPlayersData,
    RequestProblem,
    Answer(usize),
    Fire(Fire),
}

