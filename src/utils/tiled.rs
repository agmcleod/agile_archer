extern crate gfx;
extern crate tiled;

use std::collections::HashMap;
use renderer;
use renderer::tiled::{TileMapPlane, PlaneRenderer};

const COLLISION_LAYERS: [&str; 1] = ["ground"];

fn for_each_cell<F>(layer: &tiled::Layer, include_zero: bool, mut cb: F)
    where F: FnMut(usize, usize)
{
    for (y, cols) in layer.tiles.iter().enumerate() {
        for (x, cell) in cols.iter().enumerate() {
            if include_zero || *cell != 0 {
                cb(x, y);
            }
        }
    }
}

pub fn parse_out_map_layers<R, F>(
    map: &tiled::Map,
    tiles_texture: &gfx::handle::ShaderResourceView<R, [f32; 4]>,
    factory: &mut F,
    target: &renderer::WindowTargets<R>) -> (Vec<PlaneRenderer<R>>, Vec<HashMap<usize, Vec<usize>>>, HashMap<usize, Vec<usize>>)
    where R: gfx::Resources, F: gfx::Factory<R>
{
    let mut tile_map_render_data: Vec<PlaneRenderer<R>> = Vec::new();
    let mut ground_tiles: HashMap<i32, Vec<i32>> = HashMap::new();
    let mut unpassable_tiles: HashMap<usize, Vec<usize>> = HashMap::new();
    for layer in map.layers.iter() {
        if layer.name != "meta" {
            let tilemap_plane = TileMapPlane::new(&map, &layer);
            tile_map_render_data.push(PlaneRenderer::new(factory, &tilemap_plane, tiles_texture, target));
            if COLLISION_LAYERS.contains(&layer.name.as_ref()) {
                for_each_cell(&layer, false, |x, y| {
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
                                // it is open, so let's add it
                                // we track x by y instead of y by x, as we need to go in that order for the tile grouping of grounds
                                let x = x as i32;
                                let y = (y - 1) as i32;
                                if ground_tiles.contains_key(&x) {
                                    let mut ys = ground_tiles.get_mut(&x).unwrap();
                                    ys.push(y);
                                } else {
                                    ground_tiles.insert(x, vec![y]);
                                }
                            }
                        } else {
                            let x = x as i32;
                            let y = (y - 1) as i32;
                            if ground_tiles.contains_key(&x) {
                                let mut ys = ground_tiles.get_mut(&x).unwrap();
                                ys.push(y);
                            } else {
                                ground_tiles.insert(x, vec![y]);
                            }
                        }
                    }
                });
            }
        }
    }

    // we will store these Y by X to make it easier to work with the other data.
    let mut groups: Vec<Vec<(i32, i32)>> = Vec::new();

    for (col, rows) in ground_tiles.iter() {
        for row in rows {
            let mut found = false;
            let mut temp_row = 0;
            let mut temp_col = 0;
            let mut target_group_index = 0;

            for (i, group) in groups.iter().enumerate() {
                let last_group = &group[group.len() - 1];
                // if the X is 0 or +1 to the right. If the Y is between +1 and -1 from the last
                if col - last_group.1 >= 0i32 && row - last_group.0 < 2i32 && row - last_group.0 > -2i32 {
                    temp_row = *row;
                    temp_col = *col;
                    target_group_index = i;
                    found = true;
                    break
                }
            }
            if found {
                groups.get_mut(target_group_index).unwrap().push((temp_row, temp_col));
            } else {
                groups.push(vec![(*row, *col)]);
            }
        }
    }

    let mut hash_groups: Vec<HashMap<usize, Vec<usize>>> = Vec::new();

    for group in groups {
        let mut coords: HashMap<usize, Vec<usize>> = HashMap::new();
        for (y, x) in group {
            let y = y as usize;
            let x = x as usize;
            if coords.contains_key(&y) {
                let mut xs = coords.get_mut(&y).unwrap();
                xs.push(x);
            } else {
                coords.insert(y, vec![x]);
            }
        }

        hash_groups.push(coords);
    }

    (
        tile_map_render_data,
        hash_groups,
        unpassable_tiles
    )
}