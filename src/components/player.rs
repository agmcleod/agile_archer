extern crate specs;

use specs::Component;

pub struct Player {
    pub moving: bool,
    pub movement_route: Vec<(usize, usize)>,
}

impl Player {
    pub fn new() -> Player {
        Player{
            moving: false,
            movement_route: Vec::new(),
        }
    }
}

impl Component for Player {
    type Storage = specs::HashMapStorage<Player>;
}