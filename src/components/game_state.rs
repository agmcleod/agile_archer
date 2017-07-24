extern crate specs;

use specs::Component;
use types::Turn;

#[derive(Debug)]
pub struct GameState{
    pub turn: Turn,
}

impl GameState {
    pub fn new() -> GameState {
        GameState{ turn: Turn::Player }
    }
}

impl Component for GameState {
    type Storage = specs::HashMapStorage<GameState>;
}