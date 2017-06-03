extern crate specs;

use std::ops::Deref;
use specs::{Join, RunArg, System};
use components::{Input, HighlightTile, Player, Sprite, TileData, Transform};

pub struct PlayerMovement;

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

        for (_, sprite, transform) in (&highlight_tile_storage, &mut sprites, &mut transforms).join() {
            let mouse_tile_x = input.mouse_pos.0 / tile_data.tile_size[0];
            let mouse_tile_y = input.mouse_pos.1 / tile_data.tile_size[1];
            sprite.visible = false;
            if let Some(ys) = tile_data.move_to_targets.get(&(mouse_tile_x as usize)) {
                if ys.contains(&(mouse_tile_y as usize)) {
                    sprite.visible = true;
                    transform.pos.x = mouse_tile_x * tile_data.tile_size[1];
                    transform.pos.y = tile_data.map_dimensions[1] - (mouse_tile_y * tile_data.tile_size[1]) - tile_data.tile_size[1];
                }
            }
        }
    }
}