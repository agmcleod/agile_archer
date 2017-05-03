#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate specs;
extern crate cgmath;
extern crate genmesh;
extern crate tiled;

use gfx::Device;
use specs::World;

use std::path::Path;
use std::fs::File;
use tiled::parse;

mod renderer;
mod loader;
mod components;

use renderer::{ColorFormat, DepthFormat};

fn main() {
    let dim = renderer::get_dimensions();
    let builder = glutin::WindowBuilder::new()
        .with_title("Agile Archer".to_string())
        .with_dimensions(dim[0] as u32, dim[1] as u32)
        .with_vsync();
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let map_file = File::open(&Path::new("./resources/map.tmx")).unwrap();
    let map = parse(map_file).unwrap();

    let basic = renderer::Basic::new(&mut factory, encoder);
    let target = renderer::WindowTargets{
        color: main_color,
        depth: main_depth,
    };

    let mut planner = {
        let mut world = World::new();
        world.add_resource::<components::Camera>(renderer::get_ortho() as components::Camera);
        specs::Planner::<()>::new(world)
    };

    let tilemap_draw_state = renderer::tiled::TileMap::new(&map, &mut factory, dim[0] / dim[1], &target);
    let tilemap_renderer = renderer::tiled::TileMapRenderer::new(&tilemap_draw_state, &mut factory);

    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => {}
            }
        }
        window.swap_buffers().unwrap();

        tilemap_renderer.render();

        device.cleanup();
    }
}
