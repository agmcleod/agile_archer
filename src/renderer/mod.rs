extern crate gfx;
extern crate cgmath;

use cgmath::{Matrix4, Point3, Vector3};
use gfx::traits::FactoryExt;

pub mod tiled;

pub use self::tiled::*;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

pub struct WindowTargets<R: gfx::Resources> {
    pub color: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub depth: gfx::handle::DepthStencilView<R, DepthFormat>,
}

pub struct Basic<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    encoder: gfx::Encoder<R, C>,
    pso: gfx::PipelineState<R, pipe::Meta>,
}

impl<R, C> Basic<R, C>
    where R: gfx::Resources,
        C: gfx::CommandBuffer<R>
{
    pub fn new<F>(factory: &mut F, encoder: gfx::Encoder<R, C>) -> Basic<R, C>
        where F: gfx::Factory<R>
    {
        use gfx::traits::FactoryExt;

        let pso = factory.create_pipeline_simple(
            include_bytes!("shaders/basic.glslv"),
            include_bytes!("shaders/basic.glslf"),
            pipe::new()
        ).unwrap();

        Basic{
            encoder: encoder,
            pso: pso,
        }
    }
}

pub fn get_ortho() -> Matrix4<f32> {
    let dim = get_dimensions();
    let hw = dim[0] / 2.0;
    let hh = dim[1] / 2.0;
    cgmath::ortho(
        -hw, hw,
        -hh, hh,
        0.0, 1.0,
    )
}

pub fn get_view(x: f32, y: f32) -> Matrix4<f32> {
    Matrix4::look_at(
        Point3::new(x, y, 800.0),
        Point3::new(x, y, 0.0),
        Vector3::unit_y(),
    )
}

pub fn get_dimensions() -> [f32; 2] {
    [1024.0, 768.0]
}