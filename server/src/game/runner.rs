use std::collections::HashSet;
use super::*;
use super::state::GameState;
use super::event::*;
use futures::sync::mpsc::UnboundedSender;
use tokio_core::reactor::Handle;
use futures::{Future, Sink};

pub struct Runner {
    game_state: GameState,
    handle: Handle,
    output_sink: UnboundedSender<(Id, String)>,
    users: HashSet<Id>,
}

impl Runner {
    pub fn new(
        handle: Handle,
        output_sink: UnboundedSender<(Id, String)>
    ) -> Self {
        Self {
            game_state: GameState::new(),
            handle: handle,
            output_sink: output_sink,
            users: HashSet::new(),
        }
    }

    pub fn proc_event(&mut self, event: Event) {
        match event {
            Event::UserMessage(id, user_msg) => self.proc_user_event(id, user_msg)
        }
    }

    fn proc_user_event(&mut self, id: Id, user_msg: UserMessage) {
        match user_msg {
            UserMessage::Join => {
                self.game_state.player_n += 1;
                self.users.insert(id);
                self.broadcast_user_num();
            }
        }
    }

    fn broadcast_user_num(&mut self) {


        for user in self.users.iter() {
            self.output(*user, json!({
                "player_n": self.game_state.player_n
            }).to_string());
        }
    }

    fn output(&self, user: Id, msg: String) {
        self.handle.spawn(consume_result!(
            self.output_sink.clone().send((user, msg))
        ));
    }
}
