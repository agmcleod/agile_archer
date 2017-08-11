use specs::{Component, HashMapStorage};

pub struct EnergyBar;

impl EnergyBar {
    pub fn get_max_width() -> u16 {
        150
    }
}

impl Component for EnergyBar {
    type Storage = HashMapStorage<EnergyBar>;
}