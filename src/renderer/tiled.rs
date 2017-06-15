extern crate gfx;
extern crate cgmath;
extern crate genmesh;
extern crate tiled;
extern crate specs;

use cgmath::{Matrix4, Vector3, SquareMatrix};

use genmesh::{Triangulate};
use genmesh::generators::{SharedVertex, IndexedPolygon};

use tiled::{Tileset};
use specs::World;

use renderer;
use renderer::{WindowTargets};

use components;

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
        let mut vertex_data: Vec<VertexData> = Vec::new();
        let mut index_data: Vec<u32> = Vec::new();

        let mut index = 0u32;
        for (row, cols) in layer.tiles.iter().enumerate() {
            for (col, cell) in cols.iter().enumerate() {
                if *cell != 0 {
                    let x = col as f32 * tilemap.tile_width as f32;
                    let y = (tilemap.tile_height * tilemap.height) as f32 - (row as f32 * tilemap.tile_height as f32) - tilemap.tile_height as f32;
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
                            let tiles_wide = iw / (tileset.tile_width + tileset.spacing);
                            let tiles_high = ih / (tileset.tile_height + tileset.spacing);
                            let tile_width_uv = tileset.tile_width as f32 / iw as f32;
                            let tile_height_uv = tileset.tile_height as f32 / ih as f32;
                            let x = ((*cell as u32 - 1u32) % tiles_wide) as f32 + tileset.margin as f32 / iw as f32;
                            let y = ((*cell as u32 - 1u32) / tiles_wide) as f32 + tileset.margin as f32 / ih as f32;
                            let i = index as usize;
                            let tiles_wide = tiles_wide as f32;
                            let tiles_high = tiles_high as f32;
                            vertex_data[i].uv[0] = x / tiles_wide;
                            vertex_data[i].uv[1] = y / tiles_high + tile_height_uv;
                            vertex_data[i + 1].uv[0] = x / tiles_wide + tile_width_uv;
                            vertex_data[i + 1].uv[1] = y / tiles_high + tile_height_uv;
                            vertex_data[i + 2].uv[0] = x / tiles_wide + tile_width_uv;
                            vertex_data[i + 2].uv[1] = y / tiles_high;
                            vertex_data[i + 3].uv[0] = x / tiles_wide;
                            vertex_data[i + 3].uv[1] = y / tiles_high;
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

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Projection {
        model: [[f32; 4]; 4] = "u_Model",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        projection_cb: gfx::ConstantBuffer<Projection> = "b_Projection",
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::RenderTarget<renderer::ColorFormat> = "Target0",
    }
}


pub struct PlaneRenderer<R: gfx::Resources> {
    pso: gfx::PipelineState<R, pipe::Meta>,
    projection: Projection,
    params: pipe::Data<R>,
    slice: gfx::Slice<R>,
}

impl <R>PlaneRenderer<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F, tilemap_plane: &TileMapPlane, tiles_texture: &gfx::handle::ShaderResourceView<R, [f32; 4]>, target: &WindowTargets<R>) -> PlaneRenderer<R>
        where F: gfx::Factory<R>
    {
        use gfx::traits::FactoryExt;

        let pso = factory.create_pipeline_simple(
            include_bytes!("shaders/basic.glslv"),
            include_bytes!("shaders/basic.glslf"),
            pipe::new()
        ).unwrap();

        let data: Vec<Vertex> = tilemap_plane.data.iter().map(|quad| {
            Vertex{
                pos: quad.pos,
                uv: quad.uv,
            }
        }).collect();
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&data, &tilemap_plane.index_data[..]);

        PlaneRenderer{
            pso: pso,
            projection: Projection{
                model: Matrix4::identity().into(),
                proj: renderer::get_ortho().into(),
            },
            params: pipe::Data{
                vbuf: vbuf,
                projection_cb: factory.create_constant_buffer(1),
                tex: (tiles_texture.clone(), factory.create_sampler_linear()),
                out: target.color.clone(),
            },
            slice: slice,
        }
    }

    pub fn render<C>(&mut self,
        encoder: &mut gfx::Encoder<R, C>,
        world: &World)
        where R: gfx::Resources, C: gfx::CommandBuffer<R>
    {
        use std::ops::Deref;
        let camera_res = world.read_resource::<components::Camera>();
        let camera = camera_res.deref();
        self.projection.proj = (*camera).0.into();

        encoder.update_constant_buffer(&self.params.projection_cb, &self.projection);
        encoder.draw(&self.slice, &self.pso, &self.params);
    }
}