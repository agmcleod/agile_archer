extern crate specs;

use std::ops::{Deref, DerefMut};
use specs::{Fetch, FetchMut, Join, ReadStorage, WriteStorage, System};
use components::{Input, HighlightTile, Player, PlayerActionState, Sprite, TileData, Transform};
use math::astar;
use types::TileMapping;
use utils::movement;

pub struct PlayerMovement{
    pub pathable_grid: Vec<Vec<astar::TileType>>,
}

impl PlayerMovement {
    fn get_mouse_tile(&self, mouse_pos: &(i32, i32), tile_size: &[i32; 2]) -> (i32, i32) {
        (
            mouse_pos.0 / tile_size[0],
            mouse_pos.1 / tile_size[1],
        )
    }

    fn move_highlight_to_mouse(&self, mouse_tile: &(usize, usize), transform: &mut Transform, tile_data: &TileData, sprite: &mut Sprite) {
        sprite.visible = true;
        transform.pos.x = mouse_tile.0 as i32 * tile_data.tile_size[1];
        transform.pos.y = tile_data.map_dimensions[1] - (mouse_tile.1 as i32 * tile_data.tile_size[1]) - tile_data.tile_size[1];
    }

    fn astar_path_to_mouse(&self, player_transform: &Transform, tile_data: &TileData, mouse_tile: (usize, usize)) -> Vec<(usize, usize)> {
        let player_tile = tile_data.get_tile_for_world_position(&player_transform.pos);
        astar::find_path(
            &self.pathable_grid,
            player_tile,
            mouse_tile
        )
    }
}

impl<'a> System<'a> for PlayerMovement {
    type SystemData = (
        ReadStorage<'a, HighlightTile>,
        Fetch<'a, Input>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Sprite>,
        FetchMut<'a, TileData>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use std::ops::Deref;
        let (highlight_tile_storage, input_storage, mut players, mut sprites, mut tile_data_storage, mut transforms) = data;

        let input: &Input = input_storage.deref();
        let tile_data: &mut TileData = tile_data_storage.deref_mut();

        let mouse_tile = self.get_mouse_tile(&input.mouse_pos, &tile_data.tile_size);
        let mouse_tile = (mouse_tile.0 as usize, mouse_tile.1 as usize);

        let mut player_in_air = false;
        let mut player_distance = 0;
        let mut player_jump_distance = 0;

        for (player, transform) in (&mut players, &mut transforms).join() {
            player_in_air = player.in_air();
            player_jump_distance = player.jump_distance;
            let player_tile = tile_data.get_tile_for_world_position(&transform.pos);
            player_distance = movement::distance_to_tile(&player_tile, &mouse_tile);
            if input.mouse_pressed && !player.moving() && !player.jumping() {
                if player.in_air() && player_distance <= player.jump_distance {
                    for (i, group) in tile_data.walkable_groups.iter().enumerate() {
                        if group.contains(&mouse_tile.1, &mouse_tile.0) {
                            player.action_state = PlayerActionState::Moving;
                            player.movement_route = self.astar_path_to_mouse(&transform, &tile_data, mouse_tile);
                            tile_data.player_group_index = i;
                            break
                        }
                    }
                } else {
                    let group = &tile_data.walkable_groups[tile_data.player_group_index];
                    let mut found = false;
                    if group.contains(&mouse_tile.1, &mouse_tile.0) {
                        player.action_state = PlayerActionState::Moving;
                        found = true;
                    } else if tile_data.jump_targets.contains(&mouse_tile.1, &mouse_tile.0) {
                        player.action_state = PlayerActionState::Jumping;
                        found = true;
                    }

                    if found {
                        player.movement_route = self.astar_path_to_mouse(&transform, &tile_data, mouse_tile);
                    }
                }
            } else if player.moving() || player.jumping() {
                // will need to track this differently to lerp at somepoint
                let mut done = false;
                if let Some(next_pos) = player.movement_route.iter().next() {
                    transform.pos.x = next_pos.0 as i32 * tile_data.tile_size[0];
                    transform.pos.y = tile_data.map_dimensions[1] - (next_pos.1 as i32 * tile_data.tile_size[1]) - tile_data.tile_size[1];
                } else {
                    if player.moving() {
                        player.action_state = PlayerActionState::OnGround;
                    } else if player.jumping() {
                        player.action_state = PlayerActionState::InAir;
                    }
                    done = true;
                }

                // passed through a position
                if !done {
                    player.movement_route.remove(0);
                }
            }
        }

        for (_, mut sprite, mut transform) in (&highlight_tile_storage, &mut sprites, &mut transforms).join() {
            sprite.visible = false;
            if player_in_air && player_distance <= player_jump_distance {
                for group in &tile_data.walkable_groups {
                    if group.contains(&mouse_tile.1, &mouse_tile.0) {
                        self.move_highlight_to_mouse(&mouse_tile, &mut transform, &tile_data, &mut sprite);
                    }
                }
            } else if !player_in_air {
                let group = &tile_data.walkable_groups[tile_data.player_group_index];
                if group.contains(&mouse_tile.1, &mouse_tile.0) {
                    self.move_highlight_to_mouse(&mouse_tile, &mut transform, &tile_data, &mut sprite);
                } else if player_distance <= player_jump_distance && tile_data.jump_targets.contains(&mouse_tile.1, &mouse_tile.0) {
                    self.move_highlight_to_mouse(&mouse_tile, &mut transform, &tile_data, &mut sprite);
                }
            }
        }
    }
}