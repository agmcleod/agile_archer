extern crate gfx;
extern crate cgmath;
extern crate specs;

use specs::World;
use cgmath::{SquareMatrix, Matrix4, Point3, Vector3};
use gfx::traits::FactoryExt;

use components;
pub mod tiled;

pub use self::tiled::*;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

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
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

pub struct WindowTargets<R: gfx::Resources> {
    pub color: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub depth: gfx::handle::DepthStencilView<R, DepthFormat>,
}

pub struct Basic<R: gfx::Resources> {
    pso: gfx::PipelineState<R, pipe::Meta>,
    projection: Projection,
    target: WindowTargets<R>,
}

impl<R> Basic<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F, target: WindowTargets<R>) -> Basic<R>
        where F: gfx::Factory<R>
    {
        use gfx::traits::FactoryExt;

        let pso = factory.create_pipeline_simple(
            include_bytes!("shaders/basic.glslv"),
            include_bytes!("shaders/basic.glslf"),
            pipe::new()
        ).unwrap();

        Basic{
            pso: pso,
            projection: Projection{
                model: Matrix4::identity().into(),
                proj: get_ortho().into(),
            },
            target: target,
        }
    }

    pub fn render_map<C, F>(&mut self,
        encoder: &mut gfx::Encoder<R, C>,
        world: &World,
        tilemap_planes: &Vec<TileMapPlane>,
        tiles_texture: &gfx::handle::ShaderResourceView<R, [f32; 4]>,
        factory: &mut F)
        where R: gfx::Resources, C: gfx::CommandBuffer<R>, F: gfx::Factory<R>
    {

        let camera = world.read_resource::<components::Camera>().wait();
        {
            encoder.clear(&self.target.color, [16.0 / 256.0, 14.0 / 256.0, 22.0 / 256.0, 1.0]);
            encoder.clear_depth(&self.target.depth, 1.0);
        }

        for tilemap_plane in tilemap_planes.iter() {
            let data: Vec<Vertex> = tilemap_plane.data.iter().map(|quad| {
                Vertex{
                    pos: quad.pos,
                    uv: quad.uv,
                }
            }).collect();
            let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&data, &tilemap_plane.index_data[..]);
            let mut params = pipe::Data{
                vbuf: vbuf,
                projection_cb: factory.create_constant_buffer(1),
                tex: (tiles_texture.clone(), factory.create_sampler_linear()),
                out: self.target.color.clone(),
            };

            self.projection.proj = (*camera).0.into();

            encoder.update_constant_buffer(&params.projection_cb, &self.projection);
            encoder.draw(&slice, &self.pso, &params);
        }
}
}

pub fn get_ortho() -> Matrix4<f32> {
    let dim = get_dimensions();
    cgmath::ortho(
        0.0, dim[0],
        0.0, dim[1],
        0.0, 1.0,
    )
}

pub fn get_dimensions() -> [f32; 2] {
    [960.0, 640.0]
}