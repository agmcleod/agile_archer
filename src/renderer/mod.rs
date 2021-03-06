use cgmath::{ortho, Matrix4};
use gfx;

mod tiled;
mod basic;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub use self::basic::*;
pub use self::tiled::*;

pub fn get_ortho() -> Matrix4<f32> {
    let dim = get_dimensions();
    ortho(
        0.0, dim[0],
        0.0, dim[1],
        0.0, 1.0,
    )
}

pub fn get_dimensions() -> [f32; 2] {
    [960.0, 640.0]
}