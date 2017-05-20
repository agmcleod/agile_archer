extern crate gfx;
extern crate cgmath;
extern crate genmesh;
extern crate tiled;
extern crate specs;

use renderer;
use loader;
use specs::World;

use cgmath::{SquareMatrix, Matrix4, Vector3};

use gfx::traits::FactoryExt;
use genmesh::{Vertices, Triangulate};
use genmesh::generators::{Plane, SharedVertex, IndexedPolygon};

use tiled::{Tileset};
use components;

// this is a value based on a max buffer size (and hence tilemap size) of 64x64
// I imagine you would have a max buffer length, with multiple TileMap instances
// of varying sizes based on current screen resolution
pub const TILEMAP_BUF_LENGTH: usize = 4096;

// Actual tilemap data that makes up the elements of the UBO.
// NOTE: It may be a bug, but it appears that
// [f32;2] won't work as UBO data. Possibly an issue with
// binding generation
gfx_defines!{
    constant TileMapData {
        data: [f32; 4] = "data",
    }

    constant ProjectionStuff {
        model: [[f32; 4]; 4] = "u_Model",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    constant TilemapStuff {
        world_size: [f32; 4] = "u_WorldSize",
        tilesheet_size: [f32; 4] = "u_TilesheetSize",
        offsets: [f32; 2] = "u_TileOffsets",
    }

    vertex VertexData {
        pos: [f32; 3] = "a_Pos",
        buf_pos: [f32; 2] = "a_BufPos",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<VertexData> = (),
        projection_cb: gfx::ConstantBuffer<ProjectionStuff> = "b_VsLocals",
        // tilemap stuff
        tilemap: gfx::ConstantBuffer<TileMapData> = "b_TileMap",
        tilemap_cb: gfx::ConstantBuffer<TilemapStuff> = "b_PsLocals",
        tilesheet: gfx::TextureSampler<[f32; 4]> = "t_TileSheet",
        // output
        out_color: gfx::RenderTarget<gfx::format::Rgba8> = "Target0",
        out_depth: gfx::DepthTarget<gfx::format::DepthStencil> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl TileMapData {
    pub fn new_empty() -> TileMapData {
        TileMapData { data: [0.0, 0.0, 0.0, 0.0] }
    }
    pub fn new(data: [f32; 4]) -> TileMapData {
        TileMapData { data: data }
    }
}

pub struct TileMapPlane<R: gfx::Resources> {
    pub params: pipe::Data<R>,
    pub texture_data: Vec<TileMapData>,
    pub slice: gfx::Slice<R>,
    proj_stuff: ProjectionStuff,
    proj_dirty: bool,
    tm_stuff: TilemapStuff,
    tm_dirty: bool,
    pub data: Vec<TileMapData>,
}

impl<R> TileMapPlane<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F, tilemap: &tiled::Map, layer: &tiled::Layer, target: &renderer::WindowTargets<R>) -> TileMapPlane<R>
        where F: gfx::Factory<R>
    {
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
                        pos: [x, y, 0.0],
                        buf_pos: [col as f32, row as f32],
                    });
                    vertex_data.push(VertexData{
                        pos: [x + w, y, 0.0],
                        buf_pos: [col as f32, row as f32],
                    });
                    vertex_data.push(VertexData{
                        pos: [x + w, y + h, 0.0],
                        buf_pos: [col as f32, row as f32],
                    });
                    vertex_data.push(VertexData{
                        pos: [x, y + h, 0.0],
                        buf_pos: [col as f32, row as f32],
                    });

                    index_data.push(index);
                    index_data.push(index + 1);
                    index_data.push(index + 2);
                    index_data.push(index + 2);
                    index_data.push(index + 3);
                    index_data.push(index);

                    index += 4;
                }
            }
        }

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, &index_data[..]);

        let tileset = tilemap.tilesets.get(0).unwrap(); // working under the assumption i will only use one tileset
        let image = tileset.images.get(0).unwrap();
        let tiles_texture = loader::gfx_load_texture(format!("./resources/{}", image.source).as_ref(), factory);

        let params = pipe::Data {
            vbuf: vbuf,
            projection_cb: factory.create_constant_buffer(1),
            tilemap: factory.create_constant_buffer(TILEMAP_BUF_LENGTH),
            tilemap_cb: factory.create_constant_buffer(1),
            tilesheet: (tiles_texture, factory.create_sampler_linear()),
            out_color: target.color.clone(),
            out_depth: target.depth.clone(),
        };

        let mut map_data = Vec::with_capacity(total_size as usize);
        for _ in 0..total_size {
            map_data.push(TileMapData::new_empty());
        }

        let mut tiles = Vec::with_capacity((tilemap.width * tilemap.height) as usize);
        for _ in 0..(tilemap.width * tilemap.height) {
            tiles.push(TileMapData::new_empty());
        }

        for (row, cols) in layer.tiles.iter().enumerate() {
            for (col, cell) in cols.iter().enumerate() {
                if *cell != 0 {
                    for tileset in tilemap.tilesets.iter() {
                        let image = &tileset.images[0];
                        // just handling a single image for now
                        if tileset.first_gid as usize + tileset.tiles.len() - 1 <= *cell as usize {
                            let iw = image.width as u32;
                            let tiles_wide = iw / tileset.tile_width as u32;
                            let x = (*cell as u32 - 1u32) % tiles_wide;
                            let y = (*cell as u32 - 1u32) / tiles_wide;
                            let idx = (y * tilemap.width) + x;
                            tiles[idx as usize] = TileMapData::new([x as f32, y  as f32, 0.0, 0.0]);
                            break
                        }
                    }
                }
            }
        }

        TileMapPlane{
            texture_data: tiles,
            slice: slice,
            params: params,
            proj_stuff: ProjectionStuff {
                model: Matrix4::identity().into(),
                proj: renderer::get_ortho().into(),
            },
            proj_dirty: true,
            tm_stuff: TilemapStuff{
                world_size: [tilemap.width as f32, tilemap.height as f32, tilemap.tile_width as f32, 0.0],
                tilesheet_size: [
                    (tileset.images[0].width / tileset.tile_width as i32) as f32,
                    (tileset.images[0].height / tileset.tile_height as i32) as f32,
                    tileset.images[0].width as f32,
                    tileset.images[0].height as f32
                ],
                offsets: [0.0, 0.0],
            },
            tm_dirty: true,
            data: map_data,
        }
    }

    fn prepare_buffers<C>(&self, encoder: &mut gfx::Encoder<R, C>, update_data: bool) where C: gfx::CommandBuffer<R> {
        if update_data {
            encoder.update_buffer(&self.params.tilemap, &self.data, 0).unwrap();
        }
        if self.proj_dirty {
            encoder.update_constant_buffer(&self.params.projection_cb, &self.proj_stuff);
        }
        if self.tm_dirty {
            encoder.update_constant_buffer(&self.params.tilemap_cb, &self.tm_stuff);
        }
    }

    pub fn update_view(&mut self, view: &Matrix4<f32>) {
        self.proj_stuff.proj = view.clone().into();
        self.proj_dirty = true;
    }

    pub fn update_x_offset(&mut self, amt: f32) {
        self.tm_stuff.offsets[0] = amt;
        self.tm_dirty = true;
    }

    pub fn update_y_offset(&mut self, amt: f32) {
        self.tm_stuff.offsets[1] = amt;
        self.tm_dirty = true;
    }
}

pub fn populate_tilemap<R>(tilemap: &mut TileMapRenderer<R>, map_data: &tiled::Map)
    where R: gfx::Resources
{
    let layers = &map_data.layers;
    for layer in layers {

    }
}

pub struct TileMapRenderer<R: gfx::Resources> {
    tilemap_planes: Vec<TileMapPlane<R>>,
    tile_size: f32,
    tilemap_size: [usize; 2],
    charmap_size: [usize; 2],
    limit_coords: [usize; 2],
    focus_coords: [usize; 2],
    focus_dirty: bool,
    projection: gfx::handle::Buffer<R, ProjectionStuff>,
    pso: gfx::PipelineState<R, pipe::Meta>,
}

impl<R> TileMapRenderer<R>
    where R: gfx::Resources
{
    pub fn new<F>(map: &tiled::Map, factory: &mut F, target: &renderer::WindowTargets<R>) -> TileMapRenderer<R>
        where F: gfx::Factory<R>
    {
        let vert_src = include_bytes!("shaders/tilemap.glslv");
        let frag_src = include_bytes!("shaders/tilemap.glslf");

        let tilemap_planes = map.layers.iter().map(|layer| {
            TileMapPlane::new(
                factory, map, layer, target
            )
        }).collect::<Vec<TileMapPlane<R>>>();

        TileMapRenderer {
            tilemap_planes: tilemap_planes,
            tile_size: map.tile_width as f32,
            tilemap_size: [map.width as usize, map.height as usize],
            charmap_size: [map.width as usize, map.height as usize],
            limit_coords: [0, 0],
            focus_coords: [0, 0],
            focus_dirty: false,
            projection: factory.create_constant_buffer(1),
            pso: factory.create_pipeline_simple(vert_src, frag_src, pipe::new()).unwrap(),
        }
    }

    pub fn set_focus(&mut self, focus: [usize; 2]) {
        if focus[0] <= self.limit_coords[0] && focus[1] <= self.limit_coords[1] {
            self.focus_coords = focus;
            let mut charmap_ypos = 0;
            for ypos in self.focus_coords[1] .. self.focus_coords[1]+self.charmap_size[1] {
                let mut charmap_xpos = 0;
                for xpos in self.focus_coords[0] .. self.focus_coords[0]+self.charmap_size[0] {
                    let tile_idx = (ypos * self.tilemap_size[0]) + xpos;
                    let charmap_idx = (charmap_ypos * self.charmap_size[0]) + charmap_xpos;
                    for tilemap_plane in self.tilemap_planes.iter_mut() {
                        tilemap_plane.data[charmap_idx] = tilemap_plane.texture_data[tile_idx];
                    }
                    charmap_xpos += 1;
                }
                charmap_ypos += 1;
            }
            self.focus_dirty = true;
        } else {
            panic!("tried to set focus to {:?} with tilemap_size of {:?}", focus, self.tilemap_size);
        }
    }

    pub fn render<C>(&mut self, encoder: &mut gfx::Encoder<R, C>, world: &World)
        where R: gfx::Resources, C: gfx::CommandBuffer<R>
    {

        let camera = world.read_resource::<components::Camera>().wait();
        {
            let params = &self.tilemap_planes[0].params;
            encoder.clear(&params.out_color, [16.0 / 256.0, 14.0 / 256.0, 22.0 / 256.0, 1.0]);
            encoder.clear_depth(&params.out_depth, 1.0);
        }
        for tilemap_plane in self.tilemap_planes.iter_mut() {
            tilemap_plane.update_view(&(*camera).0.into());
            tilemap_plane.prepare_buffers(encoder, self.focus_dirty);
            encoder.draw(&tilemap_plane.slice, &self.pso, &tilemap_plane.params);
        }
    }
}