extern crate specs;
extern crate tiled;
use specs::Component;
use std::collections::HashMap;
use tiled::Map;
use cgmath::Vector2;
use types::TileMapping;

#[derive(Debug)]
pub struct TileData {
    pub walkable_groups: Vec<TileMapping<usize>>,
    pub map_size: [i32; 2],
    pub map_dimensions: [i32; 2],
    pub tile_size: [i32; 2],
    pub player_group_index: usize,
    pub jump_targets: TileMapping<usize>,
}

impl TileData {
    pub fn new(walkable_groups: Vec<TileMapping<usize>>, map: &Map, jump_targets: TileMapping<usize>) -> TileData {
        TileData{
            walkable_groups: walkable_groups,
            map_size: [map.width as i32, map.height as i32],
            map_dimensions: [(map.width * map.tile_width) as i32, (map.height * map.tile_height) as i32],
            tile_size: [map.tile_width as i32, map.tile_height as i32],
            player_group_index: 0,
            jump_targets: jump_targets,
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

    pub fn get_tile_for_world_position(&self, position: &Vector2<i32>) -> (usize, usize) {
        let mut player_tile = ((position.x / &self.tile_size[0]) as usize, (position.y / &self.tile_size[1]) as usize);
        player_tile.1 = self.map_size[1] as usize - player_tile.1;
        player_tile
    }
}

impl Component for TileData {
    type Storage = specs::HashMapStorage<TileData>;
}