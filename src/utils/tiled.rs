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
    target: &renderer::WindowTargets<R>) -> (Vec<PlaneRenderer<R>>, Vec<Vec<(usize, usize)>>, HashMap<usize, Vec<usize>>)
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
                            if xs.contains(&x) {
                                // it is open, so let's add it
                                // we track x by y instead of y by x, as we need to go in that order for the tile grouping of grounds
                                let x = x as i32;
                                if ground_tiles.contains_key(&x) {
                                    let mut ys = ground_tiles.get_mut(&x).unwrap();
                                    ys.push(y as i32);
                                } else {
                                    ground_tiles.insert(x, vec![y as i32]);
                                }
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
            for group in groups.iter_mut() {
                let last_group = &group[group.len() - 1];
                // if the X is 0 or +1 to the right. If the Y is between +1 and -1 from the last
                if col - last_group.1 >= 0i32 && row - last_group.0 < 2i32 && row - last_group.0 > -2i32 {
                    group.push((*row, *col));
                    found = true;
                    break
                }
            }
            if !found {
                groups.push(vec![(*row, *col)]);
            }
        }
    }

    (
        tile_map_render_data,
        groups,
        unpassable_tiles
    )
}