extern crate specs;

use specs::{VecStorage, Component};

pub struct Enemy;

impl Component for Enemy {
    type Storage = VecStorage<Enemy>;
}