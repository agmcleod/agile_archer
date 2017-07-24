use std::ops::Deref;
use specs::{Fetch, FetchMut, Join, WriteStorage, System};
use components::{GameState, Player};
use types::Turn;

pub struct ProcessTurn{

}

impl<'a> System<'a> for ProcessTurn {
    type SystemData = (
        Fetch<'a, GameState>,
        WriteStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (game_state_storage, mut player_storage) = data;
        let game_state: &GameState = game_state_storage.deref();

        if game_state.turn == Turn::Player {

        }
    }
}