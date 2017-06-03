extern crate specs;

use specs::Component;

#[derive(Debug)]
pub struct HighlightTile;

impl Component for HighlightTile {
    type Storage = specs::HashMapStorage<HighlightTile>;
}