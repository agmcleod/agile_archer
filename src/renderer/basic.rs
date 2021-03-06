extern crate gfx;
extern crate cgmath;
extern crate specs;

use specs::World;
use renderer::{ColorFormat, DepthFormat};
use cgmath::{SquareMatrix, Matrix4, Vector3};
use gfx::traits::FactoryExt;
use gfx::texture;
use components;
use spritesheet::{Frame, Spritesheet};
use renderer::get_ortho;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        color: [f32; 4] = "a_Color",
    }

    constant Projection {
        model: [[f32; 4]; 4] = "u_Model",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        projection_cb: gfx::ConstantBuffer<Projection> = "b_Projection",
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

#[derive(Clone)]
pub struct WindowTargets<R: gfx::Resources> {
    pub color: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub depth: gfx::handle::DepthStencilView<R, DepthFormat>,
}

pub struct Basic<R: gfx::Resources> {
    pso: gfx::PipelineState<R, pipe::Meta>,
    projection: Projection,
    target: WindowTargets<R>,
    color_texture: (gfx::handle::ShaderResourceView<R, [f32; 4]>, gfx::handle::Sampler<R>),
}

impl<R> Basic<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F, target: &WindowTargets<R>) -> Basic<R>
        where F: gfx::Factory<R>
    {
        use gfx::traits::FactoryExt;

        let pso = factory.create_pipeline_simple(
            include_bytes!("shaders/basic.glslv"),
            include_bytes!("shaders/basic.glslf"),
            pipe::new()
        ).unwrap();

        let texels = [[0xff, 0xff, 0xff, 0xff]];
        let (_, texture_view) = factory.create_texture_immutable::<gfx::format::Rgba8>(
            texture::Kind::D2(1, 1, texture::AaMode::Single), &[&texels]
        ).unwrap();

        let sinfo = texture::SamplerInfo::new(
            texture::FilterMethod::Bilinear,
        texture::WrapMode::Clamp);

        Basic{
            pso: pso,
            projection: Projection{
                model: Matrix4::identity().into(),
                proj: get_ortho().into(),
            },
            target: (*target).clone(),
            color_texture: (texture_view, factory.create_sampler(sinfo)),
        }
    }

    pub fn reset_transform(&mut self) {
        self.projection.model = Matrix4::identity().into();
    }

    pub fn render<C, F>(&mut self,
        encoder: &mut gfx::Encoder<R, C>,
        world: &World,
        factory: &mut F,
        transform: &components::Transform,
        frame_name: Option<&String>,
        spritesheet: &Spritesheet,
        color: Option<[f32; 4]>,
        texture: Option<&gfx::handle::ShaderResourceView<R, [f32; 4]>>)
        where R: gfx::Resources, C: gfx::CommandBuffer<R>, F: gfx::Factory<R>
    {
        use std::ops::Deref;

        let camera_res = world.read_resource::<components::Camera>();
        let camera = camera_res.deref();
        let x = transform.pos.x as f32;
        let y = transform.pos.y as f32;
        let w = transform.size.x as f32;
        let h = transform.size.y as f32;

        let mut tx = 0.0;
        let mut ty = 0.0;
        let mut tx2 = 1.0;
        let mut ty2 = 1.0;

        if let Some(frame_name) = frame_name {
            let region = spritesheet.frames.iter().filter(|frame|
                frame.filename == *frame_name
            ).collect::<Vec<&Frame>>()[0];
            let sw = spritesheet.meta.size.w as f32;
            let sh = spritesheet.meta.size.h as f32;
            tx = region.frame.x as f32 / sw;
            ty = region.frame.y as f32 / sh;
            tx2 = (region.frame.x as f32 + region.frame.w as f32) / sw;
            ty2 = (region.frame.y as f32 + region.frame.h as f32) / sh;
        }

        let tex: (gfx::handle::ShaderResourceView<R, [f32; 4]>, gfx::handle::Sampler<R>) = if let Some(texture) = texture {
            (texture.clone(), factory.create_sampler_linear())
        } else {
            self.color_texture.clone()
        };

        let color = if let Some(color) = color {
            color
        } else {
            [1.0; 4]
        };

        let data: Vec<Vertex> = vec![
            Vertex{
                pos: [x, y],
                uv: [tx, ty2],
                color: color,
            },
            Vertex{
                pos: [x + w, y],
                uv: [tx2, ty2],
                color: color,
            },
            Vertex{
                pos: [x + w, y + h],
                uv: [tx2, ty],
                color: color,
            },
            Vertex{
                pos: [x, y + h],
                uv: [tx, ty],
                color: color,
            }
        ];

        let index_data: Vec<u32> = vec![0, 1, 2, 2, 3, 0];
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&data, &index_data[..]);

        let params = pipe::Data{
            vbuf: vbuf,
            projection_cb: factory.create_constant_buffer(1),
            tex: tex,
            out: self.target.color.clone(),
        };

        self.projection.proj = (*camera).0.into();

        encoder.update_constant_buffer(&params.projection_cb, &self.projection);
        encoder.draw(&slice, &self.pso, &params);
    }
}
