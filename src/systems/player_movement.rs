extern crate specs;

use std::ops::Deref;
use specs::{Join, RunArg, System};
use components::{Input, HighlightTile, Player, Sprite, TileData, Transform};

pub struct PlayerMovement;

impl PlayerMovement {
    fn get_mouse_tile(&self, mouse_pos: &(i32, i32), tile_size: &[i32; 2]) -> (i32, i32) {
        (
            mouse_pos.0 / tile_size[0],
            mouse_pos.1 / tile_size[1],
        )
    }
}

impl System<()> for PlayerMovement {
    fn run(&mut self, args: RunArg, _: ()) {
        let (highlight_tile_storage, input_storage, players, mut sprites, tile_data_storage, mut transforms) = args.fetch(|w| {
            (
                w.read::<HighlightTile>(),
                w.read_resource::<Input>(),
                w.read::<Player>(),
                w.write::<Sprite>(),
                w.read_resource::<TileData>(),
                w.write::<Transform>(),
            )
        });

        let input: &Input = input_storage.deref();
        let tile_data: &TileData = tile_data_storage.deref();

        let mouse_tile = self.get_mouse_tile(&input.mouse_pos, &tile_data.tile_size);

        for (_, sprite, transform) in (&highlight_tile_storage, &mut sprites, &mut transforms).join() {
            sprite.visible = false;
            if let Some(ys) = tile_data.move_to_targets.get(&(mouse_tile.0 as usize)) {
                if ys.contains(&(mouse_tile.1 as usize)) {
                    sprite.visible = true;
                    transform.pos.x = mouse_tile.0 * tile_data.tile_size[1];
                    transform.pos.y = tile_data.map_dimensions[1] - (mouse_tile.1 * tile_data.tile_size[1]) - tile_data.tile_size[1];
                }
            }
        }

        if input.mouse_pressed {
            for (_, transform) in (&players, &mut transforms).join() {
                if let Some(ys) = tile_data.move_to_targets.get(&(mouse_tile.0 as usize)) {
                    if ys.contains(&(mouse_tile.1 as usize)) {
                        transform.pos.x = mouse_tile.0 * tile_data.tile_size[1];
                        transform.pos.y = tile_data.map_dimensions[1] - (mouse_tile.1 * tile_data.tile_size[1]) - tile_data.tile_size[1];
                    }
                }
            }
        }

    }
}