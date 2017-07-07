extern crate specs;
extern crate tiled;
use specs::Component;
use std::collections::HashMap;
use tiled::Map;
use cgmath::Vector2;

#[derive(Debug)]
pub struct TileData {
    pub walkable_groups: Vec<HashMap<usize, Vec<usize>>>,
    pub map_size: [i32; 2],
    pub map_dimensions: [i32; 2],
    pub tile_size: [i32; 2],
    pub player_group_index: usize,
}

impl TileData {
    pub fn new(walkable_groups: Vec<HashMap<usize, Vec<usize>>>, map: &Map) -> TileData {
        TileData{
            walkable_groups: walkable_groups,
            map_size: [map.width as i32, map.height as i32],
            map_dimensions: [(map.width * map.tile_width) as i32, (map.height * map.tile_height) as i32],
            tile_size: [map.tile_width as i32, map.tile_height as i32],
            player_group_index: 0,
        }
    }

    pub fn set_player_group_index_from_pos(&mut self, player_pos: &Vector2<i32>) -> bool {
        let tile_y = (self.map_dimensions[1] - player_pos.y - self.tile_size[1]) / self.tile_size[1];
        let tile_x =  player_pos.x / self.tile_size[0];

        for (i, group) in self.walkable_groups.iter().enumerate() {
            if let Some(xs) = group.get(&(tile_y as usize)) {
                if xs.contains(&(tile_x as usize)) {
                    self.player_group_index = i;
                    return true
                }
            }
        }

        false
    }
}

impl Component for TileData {
    type Storage = specs::HashMapStorage<TileData>;
}