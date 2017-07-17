extern crate specs;

use specs::Component;

#[derive(PartialEq)]
pub enum PlayerActionState {
    Still,
    Moving,
    Jumping,
}

pub struct Player {
    pub action_state: PlayerActionState,
    pub movement_route: Vec<(usize, usize)>,
}

impl Player {
    pub fn new() -> Player {
        Player{
            action_state: PlayerActionState::Still,
            movement_route: Vec::new(),
        }
    }

    pub fn moving(&self) -> bool {
        self.action_state == PlayerActionState::Moving
    }
}

impl Component for Player {
    type Storage = specs::HashMapStorage<Player>;
}