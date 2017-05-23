extern crate gfx;
extern crate cgmath;
extern crate genmesh;
extern crate tiled;
extern crate specs;

use cgmath::{Matrix4, Vector3};

use genmesh::{Triangulate};
use genmesh::generators::{SharedVertex, IndexedPolygon};

use tiled::{Tileset};

pub struct VertexData {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

pub struct TileMapPlane {
    pub data: Vec<VertexData>,
    pub index_data: Vec<u32>,
}

impl TileMapPlane {
    pub fn new(tilemap: &tiled::Map, layer: &tiled::Layer) -> TileMapPlane {
        let total_size = tilemap.width * tilemap.height;
        let mut vertex_data: Vec<VertexData> = Vec::new();
        let mut index_data: Vec<u32> = Vec::new();

        let mut index = 0u32;
        for (row, cols) in layer.tiles.iter().enumerate() {
            for (col, cell) in cols.iter().enumerate() {
                if *cell != 0 {
                    let x = col as f32 * tilemap.tile_width as f32;
                    let y = row as f32 * tilemap.tile_height as f32;
                    let w = tilemap.tile_width as f32;
                    let h = tilemap.tile_height as f32;
                    vertex_data.push(VertexData{
                        pos: [x, y],
                        uv: [0.0, 0.0],
                    });
                    vertex_data.push(VertexData{
                        pos: [x + w, y],
                        uv: [0.0, 0.0],
                    });
                    vertex_data.push(VertexData{
                        pos: [x + w, y + h],
                        uv: [0.0, 0.0],
                    });
                    vertex_data.push(VertexData{
                        pos: [x, y + h],
                        uv: [0.0, 0.0],
                    });

                    index_data.push(index);
                    index_data.push(index + 1);
                    index_data.push(index + 2);
                    index_data.push(index + 2);
                    index_data.push(index + 3);
                    index_data.push(index);

                    // build out texture coord data
                    for tileset in tilemap.tilesets.iter() {
                        let image = &tileset.images[0];
                        // just handling a single image for now
                        if tileset.first_gid as usize + tileset.tiles.len() - 1 <= *cell as usize {
                            let iw = image.width as u32;
                            let ih = image.height as u32;
                            let tiles_wide = iw / tileset.tile_width as u32;
                            let tiles_high = ih / tileset.tile_width as u32;
                            let tile_width_uv = tileset.tile_width as f32 / iw as f32;
                            let tile_height_uv = tileset.tile_height as f32 / ih as f32;
                            let x = (*cell as u32 - 1u32) % tiles_wide;
                            let y = (*cell as u32 - 1u32) / tiles_wide;
                            vertex_data[index as usize].uv[0] = x as f32 / tiles_wide as f32;
                            vertex_data[index as usize].uv[1] = y as f32 / tiles_wide as f32;
                            vertex_data[index as usize + 1].uv[0] = x as f32 / tiles_wide as f32 + tile_width_uv;
                            vertex_data[index as usize + 1].uv[1] = y as f32 / tiles_wide as f32;
                            vertex_data[index as usize + 2].uv[0] = x as f32 / tiles_wide as f32 + tile_width_uv;
                            vertex_data[index as usize + 2].uv[1] = y as f32 / tiles_wide as f32 + tile_height_uv;
                            vertex_data[index as usize + 3].uv[0] = x as f32 / tiles_wide as f32;
                            vertex_data[index as usize + 3].uv[1] = y as f32 / tiles_wide as f32 + tile_height_uv;
                            break
                        }
                    }

                    index += 4;
                }
            }
        }

        TileMapPlane{
            data: vertex_data,
            index_data: index_data,
        }
    }
}