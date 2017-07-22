extern crate specs;

use specs::Component;

#[derive(PartialEq)]
pub enum PlayerActionState {
    OnGround,
    Moving,
    Jumping,
    InAir,
}

pub struct Player {
    pub action_state: PlayerActionState,
    pub movement_route: Vec<(usize, usize)>,
    pub jump_distance: usize,
}

impl Player {
    pub fn new() -> Player {
        Player{
            action_state: PlayerActionState::OnGround,
            movement_route: Vec::new(),
            jump_distance: 8,
        }
    }

    pub fn moving(&self) -> bool {
        self.action_state == PlayerActionState::Moving
    }

    pub fn jumping(&self) -> bool {
        self.action_state == PlayerActionState::Jumping
    }

    pub fn in_air(&self) -> bool {
        self.action_state == PlayerActionState::InAir
    }
}

impl Component for Player {
    type Storage = specs::HashMapStorage<Player>;
}