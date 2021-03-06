extern crate gfx;
extern crate tiled;

use linked_hash_map::LinkedHashMap;

use std::collections::HashMap;
use renderer;
use renderer::{TileMapPlane, PlaneRenderer};
use types::TileMapping;

const COLLISION_LAYERS: [&str; 1] = ["ground"];

fn for_each_cell<F>(layer: &tiled::Layer, include_zero: bool, mut cb: F)
    where F: FnMut(usize, usize, u32)
{
    for (y, cols) in layer.tiles.iter().enumerate() {
        for (x, cell) in cols.iter().enumerate() {
            if include_zero || *cell != 0 {
                cb(x, y, *cell);
            }
        }
    }
}

fn add_column_above_to_ground(x: usize, y: usize, ground_tiles: &mut TileMapping<i32>, ground_x_keys: &mut Vec<i32>) {
    // it is open, so let's add it
    // we track x by y instead of y by x, as we need to go in that order for the tile grouping of grounds
    let x = x as i32;
    let y = (y - 1) as i32;
    ground_x_keys.push(x);

    if ground_tiles.contains_key(&x) {
        let mut ys = ground_tiles.get_mut(&x).unwrap();
        ys.push(y);
    } else {
        ground_tiles.insert(x, vec![y]);
    }
}

pub fn get_map_render_data<R, F>(
    map: &tiled::Map,
    tiles_texture: &gfx::handle::ShaderResourceView<R, [f32; 4]>,
    factory: &mut F,
    target: &renderer::WindowTargets<R>) -> Vec<PlaneRenderer<R>>
    where R: gfx::Resources, F: gfx::Factory<R>
{
    map.layers.iter().map(|layer| {
        let tilemap_plane = TileMapPlane::new(&map, &layer);
        PlaneRenderer::new(factory, &tilemap_plane, tiles_texture, target)
    }).collect::<Vec<PlaneRenderer<R>>>()
}

pub fn parse_out_map_layers(map: &tiled::Map) -> (Vec<TileMapping<usize>>, TileMapping<usize>, TileMapping<usize>) {
    // stored x by y, in order to process data correctly
    let mut temp_ground_tiles: TileMapping<i32> = TileMapping(HashMap::new());
    let mut ground_x_keys: Vec<i32> = Vec::new();
    // y by x, for the rest of the game
    let mut unpassable_tiles: TileMapping<usize> = TileMapping(HashMap::new());
    for layer in map.layers.iter() {
        if COLLISION_LAYERS.contains(&layer.name.as_ref()) {
            for_each_cell(&layer, false, |x, y, _| {
                if unpassable_tiles.contains_key(&y) {
                    let mut xs = unpassable_tiles.get_mut(&y).unwrap();
                    xs.push(x);
                } else {
                    unpassable_tiles.insert(y, vec![x]);
                }
                if y > 0 {
                    // if above row for x coord is not a collision tile
                    if let Some(xs) = unpassable_tiles.get(&(y - 1)) {
                        if !xs.contains(&x) {
                            add_column_above_to_ground(x, y, &mut temp_ground_tiles, &mut ground_x_keys);
                        }
                    } else {
                        add_column_above_to_ground(x, y, &mut temp_ground_tiles, &mut ground_x_keys);
                    }
                }
            });
        }
    }

    // stored x by y
    let mut ground_tiles: LinkedHashMap<i32, Vec<i32>> = LinkedHashMap::new();
    ground_x_keys.sort();
    for key in ground_x_keys {
        ground_tiles.insert(key, temp_ground_tiles.get(&key).unwrap().clone());
    }

    // we will store these Y by X to make it easier to work with the other data.
    // storing in pair format here, so we can more easily check the last one.
    let mut walkable_coord_pairs: Vec<Vec<(i32, i32)>> = Vec::new();

    for (col, rows) in ground_tiles.iter() {
        for row in rows {
            let mut found = false;
            let mut temp_row = 0;
            let mut temp_col = 0;
            let mut target_group_index = 0;

            for (i, group) in walkable_coord_pairs.iter().enumerate() {
                let last_cell = &group[group.len() - 1];
                // if the X is 0 or +1 to the right. If the Y is between +1 and -1 from the last
                if (col - last_cell.1 == 0i32 || col - last_cell.1 == 1i32) && row - last_cell.0 < 2i32 && row - last_cell.0 > -2i32 {
                    temp_row = *row;
                    temp_col = *col;
                    target_group_index = i;
                    found = true;
                    break
                }
            }

            if found {
                walkable_coord_pairs.get_mut(target_group_index).unwrap().push((temp_row, temp_col));
            } else {
                walkable_coord_pairs.push(vec![(*row, *col)]);
            }
        }
    }

    let mut walkable_coords: Vec<TileMapping<usize>> = Vec::new();
    for pair in walkable_coord_pairs {
        let mut coords: TileMapping<usize> = TileMapping(HashMap::new());
        for (y, x) in pair {
            let y = y as usize;
            let x = x as usize;
            if coords.contains_key(&y) {
                coords.get_mut(&y).unwrap().push(x);
            } else {
                coords.insert(y, vec![x]);
            }
        }

        walkable_coords.push(coords);
    }

    let mut jump_targets: TileMapping<usize> = TileMapping(HashMap::new());

    for x in 0..(map.width as usize) {
        for y in 0..(map.height as usize) {
            if (!unpassable_tiles.contains_key(&y) || !unpassable_tiles.get(&y).unwrap().contains(&x)) &&
                (!ground_tiles.contains_key(&(x as i32)) || !ground_tiles.get(&(x as i32)).unwrap().contains(&(y as i32))) {
                if jump_targets.contains_key(&y) {
                    jump_targets.get_mut(&y).unwrap().push(x);
                } else {
                    jump_targets.insert(y, vec![x]);
                }
            }
        }
    }

    (
        walkable_coords,
        jump_targets,
        unpassable_tiles
    )
}

mod tests {
    use super::*;
    use tiled::{Map, Layer, Tileset, Image, Tile, Properties};
    #[test]

    fn test_parse_out_map_layers() {
        let image = Image{
            source: "the_image".to_string(),
            width: 16,
            height: 16,
            transparent_colour: None,
        };

        let tileset_tiles = vec![Tile{
            id: 1,
            images: vec![image],
        }];

        let layer = Layer{
            name: COLLISION_LAYERS[0].to_string(),
            opacity: 1.0,
            visible: true,
            tiles: vec![
                vec![0, 0, 0, 0, 0, 0, 0, 0],
                vec![1, 1, 1, 1, 1, 1, 0, 0],
                vec![0, 0, 0, 0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0, 0, 1, 0],
                vec![1, 1, 0, 0, 1, 1, 1, 1],
                vec![1, 1, 1, 1, 1, 1, 1, 1],
            ],
            properties: Properties::new(),
        };

        let image = Image{
            source: "the_image".to_string(),
            width: 16,
            height: 16,
            transparent_colour: None,
        };

        let map = Map{
            version: "1.0".to_string(),
            orientation: tiled::Orientation::Isometric,
            width: 8,
            height: 6,
            tile_width: 16,
            tile_height: 16,
            tilesets: vec![Tileset{ first_gid: 1, name: "Tileset".to_string(), tile_width: 16, tile_height: 16, spacing: 2, margin: 4, images: vec![image], tiles: tileset_tiles }],
            layers: vec![layer],
            object_groups: Vec::new(),
            properties: Properties::new(),
            background_colour: None,
        };
        let (groups, jump_areas, _) = parse_out_map_layers(&map);
        assert_eq!(groups.len(), 2);
        let row = groups[0].get(&(0)).unwrap();
        let test_row = [0, 1, 2, 3, 4, 5];
        assert_eq!(&row[..], &test_row[..]);

        let row = groups[1].get(&(2)).unwrap();
        let test_row = [6];
        assert_eq!(&row[..], &test_row[..]);

        let row = groups[1].get(&(3)).unwrap();
        let test_row = [0, 1, 4, 5, 7];
        assert_eq!(&row[..], &test_row[..]);

        let row = groups[1].get(&(4)).unwrap();
        let test_row = [2, 3];
        assert_eq!(&row[..], &test_row[..]);

        assert_eq!(jump_areas.get(&(0)).unwrap().len(), 2);
        assert_eq!(jump_areas.get(&(1)).unwrap().len(), 2);
        assert_eq!(jump_areas.get(&(2)).unwrap().len(), 7);
        assert_eq!(jump_areas.get(&(3)).unwrap().len(), 2);
        assert_eq!(jump_areas.contains_key(&(4)), false);
        assert_eq!(jump_areas.contains_key(&(5)), false);
    }
}