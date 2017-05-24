extern crate specs;
extern crate glutin;

use std::collections::HashMap;
use specs::Component;
use glutin::VirtualKeyCode;

pub struct Input {
    pub pressed_keys: HashMap<VirtualKeyCode, bool>,
}

impl Input {
    pub fn new(keys: Vec<VirtualKeyCode>) -> Input {
        let mut key_map: HashMap<VirtualKeyCode, bool> = HashMap::new();
        for key in keys {
            key_map.insert(key, false);
        }

        Input {
            pressed_keys: key_map,
        }
    }
}

impl Component for Input {
    type Storage = specs::HashMapStorage<Input>;
}