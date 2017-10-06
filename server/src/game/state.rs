pub struct GameState {
    pub player_n: usize,
}

impl GameState {
    pub fn new() -> Self {
        Self { player_n: 0 }
    }
}
