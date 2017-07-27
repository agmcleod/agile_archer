use std::ops::{Deref, DerefMut};
use specs::{Fetch, FetchMut, Join, ReadStorage, WriteStorage, System};
use components::{GameState, HighlightTile, Player, Sprite};
use types::Turn;

pub struct ProcessTurn;

impl<'a> System<'a> for ProcessTurn {
    type SystemData = (
        FetchMut<'a, GameState>,
        WriteStorage<'a, Player>,
        ReadStorage<'a, HighlightTile>,
        WriteStorage<'a, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut game_state_storage, mut player_storage, highlight_tile_storage, mut sprite_storage) = data;
        let game_state: &mut GameState = game_state_storage.deref_mut();

        for player in (&mut player_storage).join() {
            if game_state.turn == Turn::Player && player.energy == 0 {
                game_state.turn = Turn::Enemy;
                for (_, sprite) in (&highlight_tile_storage, &mut sprite_storage).join() {
                    if sprite.visible {
                        sprite.visible = false;
                    }
                }
            }
        }
    }
}