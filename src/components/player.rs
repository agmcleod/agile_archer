extern crate specs;

use specs::Component;

#[derive(PartialEq)]
pub enum PlayerActionState {
    OnGround,
    Moving,
    Jumping,
    InAir,
}

const BASE_ENERGY: usize = 10;

pub struct Player {
    pub action_state: PlayerActionState,
    pub movement_route: Vec<(usize, usize)>,
    pub jump_distance: usize,
    pub energy: usize,
}

impl Player {
    pub fn new() -> Player {
        Player{
            action_state: PlayerActionState::OnGround,
            movement_route: Vec::new(),
            jump_distance: 8,
            energy: BASE_ENERGY,
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

    pub fn reset_energy(&mut self) {
        self.energy = BASE_ENERGY;
    }
}

impl Component for Player {
    type Storage = specs::HashMapStorage<Player>;
}