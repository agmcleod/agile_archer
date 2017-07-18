extern crate specs;

use std::ops::Deref;
use specs::{Fetch, Join, ReadStorage, WriteStorage, System};
use components::{Input, HighlightTile, Player, PlayerActionState, Sprite, TileData, Transform};
use math::astar;
use types::TileMapping;

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
}

impl<'a> System<'a> for PlayerMovement {
    type SystemData = (
        ReadStorage<'a, HighlightTile>,
        Fetch<'a, Input>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Sprite>,
        Fetch<'a, TileData>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use std::ops::Deref;
        let (highlight_tile_storage, input_storage, mut players, mut sprites, tile_data_storage, mut transforms) = data;

        let input: &Input = input_storage.deref();
        let tile_data: &TileData = tile_data_storage.deref();

        let mouse_tile = self.get_mouse_tile(&input.mouse_pos, &tile_data.tile_size);
        let mouse_tile = (mouse_tile.0 as usize, mouse_tile.1 as usize);

        for (_, sprite, transform) in (&highlight_tile_storage, &mut sprites, &mut transforms).join() {
            sprite.visible = false;
            let group = &tile_data.walkable_groups[tile_data.player_group_index];
            if group.contains(&mouse_tile.1, &mouse_tile.0) {
                sprite.visible = true;
                transform.pos.x = mouse_tile.0 as i32 * tile_data.tile_size[1];
                transform.pos.y = tile_data.map_dimensions[1] - (mouse_tile.1 as i32 * tile_data.tile_size[1]) - tile_data.tile_size[1];
            }

            if tile_data.jump_targets.contains(&mouse_tile.1, &mouse_tile.0) {
                sprite.visible = true;
                transform.pos.x = mouse_tile.0 as i32 * tile_data.tile_size[1];
                transform.pos.y = tile_data.map_dimensions[1] - (mouse_tile.1 as i32 * tile_data.tile_size[1]) - tile_data.tile_size[1];
            }
        }

        for (player, transform) in (&mut players, &mut transforms).join() {
            if input.mouse_pressed && !player.moving() && !player.jumping() {
                let group = &tile_data.walkable_groups[tile_data.player_group_index];
                let mut target_selected = false;
                if group.contains(&mouse_tile.1, &mouse_tile.1) {
                    target_selected = true;
                    player.action_state = PlayerActionState::Moving;
                } else if tile_data.jump_targets.contains(&mouse_tile.1, &mouse_tile.0) {
                    player.action_state = PlayerActionState::Jumping;
                    target_selected = true;
                }
                player.movement_route = astar::find_path(
                    &self.pathable_grid, ((transform.pos.x / tile_data.tile_size[0]) as usize,
                    (tile_data.map_size[1] - transform.pos.y / tile_data.tile_size[1]) as usize),
                    mouse_tile
                );
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
    }
}