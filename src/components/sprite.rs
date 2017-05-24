extern crate specs;

use specs::{VecStorage, Component};

pub struct Sprite {
    pub texture_name: String,
    pub frame_name: String,
}

impl Component for Sprite {
    type Storage = VecStorage<Sprite>;
}