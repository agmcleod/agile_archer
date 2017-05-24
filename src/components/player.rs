extern crate specs;

use specs::Component;

pub struct Player {

}

impl Component for Player {
    type Storage = specs::HashMapStorage<Player>;
}