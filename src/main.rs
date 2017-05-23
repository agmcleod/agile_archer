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
use renderer::tiled::TileMapPlane;

fn main() {
    let dim = renderer::get_dimensions();
    let builder = glutin::WindowBuilder::new()
        .with_title("Agile Archer".to_string())
        .with_dimensions(dim[0] as u32, dim[1] as u32)
        .with_vsync();
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let map_file = File::open(&Path::new("./resources/map.tmx")).unwrap();
    let map = parse(map_file).unwrap();

    let target = renderer::WindowTargets{
        color: main_color,
        depth: main_depth,
    };

    let mut basic = renderer::Basic::new(&mut factory, target);

    let mut planner = {
        let mut world = World::new();
        world.add_resource::<components::Camera>(components::Camera(renderer::get_ortho()));
        specs::Planner::<()>::new(world)
    };

    let tile_map_render_data: Vec<TileMapPlane> = map.layers.iter().map(|layer| {
        TileMapPlane::new(&map, &layer)
    }).collect();

    let tileset = map.tilesets.get(0).unwrap(); // working under the assumption i will only use one tileset
    let image = tileset.images.get(0).unwrap();
    let tiles_texture = loader::gfx_load_texture(format!("./resources/{}", image.source).as_ref(), &mut factory);

    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => {}
            }
        }

        basic.render_map(&mut encoder, planner.mut_world(), &tile_map_render_data, &tiles_texture, &mut factory);

        encoder.flush(&mut device);

        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
