#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate specs;
extern crate cgmath;
extern crate genmesh;
extern crate tiled;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use gfx::Device;
use specs::{Gate, Join, World};

use std::path::Path;
use std::fs::File;
use tiled::parse;
use glutin::{ElementState, VirtualKeyCode};

mod renderer;
mod loader;
mod components;
mod spritesheet;

use components::{Camera, Input, Player, Sprite, Transform};

use renderer::{ColorFormat, DepthFormat};
use renderer::tiled::TileMapPlane;

use spritesheet::Spritesheet;

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
        world.add_resource::<Camera>(Camera(renderer::get_ortho()));
        world.add_resource::<Input>(Input::new(vec![VirtualKeyCode::W, VirtualKeyCode::A, VirtualKeyCode::S, VirtualKeyCode::D]));
        world.register::<Sprite>();
        world.register::<Transform>();
        world.register::<Player>();
        world.create_now().with(Transform::new(0, 0, 32, 64, 0.0, 1.0, 1.0)).with(Sprite{ frame_name: String::from("player.png") }).with(Player{});
        specs::Planner::<()>::new(world)
    };

    let tile_map_render_data: Vec<TileMapPlane> = map.layers.iter().map(|layer| {
        TileMapPlane::new(&map, &layer)
    }).collect();

    let tileset = map.tilesets.get(0).unwrap(); // working under the assumption i will only use one tileset
    let image = tileset.images.get(0).unwrap();
    let tiles_texture = loader::gfx_load_texture(format!("./resources/{}", image.source).as_ref(), &mut factory);

    let asset_data = loader::read_text_from_file("./resources/assets.json").unwrap();
    let spritesheet: Spritesheet = serde_json::from_str(asset_data.as_ref()).unwrap();
    let asset_texture = loader::gfx_load_texture("./resources/assets.png", &mut factory);

    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) | glutin::Event::Closed => break 'main,
                glutin::Event::KeyboardInput(key_state, _, key) => {
                    let world = planner.mut_world();
                    let mut input = world.write_resource::<Input>().wait();
                    let key = key.unwrap();
                    if input.pressed_keys.contains_key(&key) {
                        match key_state {
                            ElementState::Pressed => input.pressed_keys.insert(key, true),
                            ElementState::Released => input.pressed_keys.insert(key, false),
                        };
                    }
                },
                _ => {}
            }
        }

        basic.reset_transform();

        basic.render_map(&mut encoder, planner.mut_world(), &tile_map_render_data, &tiles_texture, &mut factory);

        let world = planner.mut_world();
        let sprites = world.read::<Sprite>().pass();
        let transforms = world.read::<Transform>().pass();

        for (sprite, transform) in (&sprites, &transforms).join() {
            basic.render(&mut encoder, world, &mut factory, &transform, &sprite, &spritesheet, &asset_texture);
        }

        encoder.flush(&mut device);

        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
