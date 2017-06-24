extern crate specs;
extern crate tiled;
use specs::Component;
use std::collections::HashMap;
use tiled::Map;

#[derive(Debug)]
pub struct TileData {
    pub walkable_groups: Vec<HashMap<usize, Vec<usize>>>,
    pub map_size: [i32; 2],
    pub map_dimensions: [i32; 2],
    pub tile_size: [i32; 2],
}

impl TileData {
    pub fn new(walkable_groups: Vec<HashMap<usize, Vec<usize>>>, map: &Map) -> TileData {
        TileData{
            walkable_groups: walkable_groups,
            map_size: [map.width as i32, map.height as i32],
            map_dimensions: [(map.width * map.tile_width) as i32, (map.height * map.tile_height) as i32],
            tile_size: [map.tile_width as i32, map.tile_height as i32],
        }
    }
}

impl Component for TileData {
    type Storage = specs::HashMapStorage<TileData>;
}