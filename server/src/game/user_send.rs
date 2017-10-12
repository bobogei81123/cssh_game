use super::data_struct::*;

#[derive(Deserialize, Debug)]
pub enum UserSend {
    Join,
    Fire(Fire),
    RequestSyncGameState,
    RequestProblem,
}

