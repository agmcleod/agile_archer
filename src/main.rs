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

use std::collections::HashMap;

use gfx::Device;
use specs::{Gate, Join, Planner, World};

use std::path::Path;
use std::fs::File;
use tiled::parse;
use glutin::{ElementState, VirtualKeyCode};

mod renderer;
mod loader;
mod components;
mod spritesheet;
mod systems;

use components::{Camera, HighlightTile, Input, Player, Sprite, TileData, Transform};

use renderer::{ColorFormat, DepthFormat};
use renderer::tiled::{TileMapPlane, PlaneRenderer};

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

    let mut basic = renderer::Basic::new(&mut factory, &target);

    let tileset = map.tilesets.get(0).unwrap(); // working under the assumption i will only use one tileset
    let image = tileset.images.get(0).unwrap();
    let tiles_texture = loader::gfx_load_texture(format!("./resources/{}", image.source).as_ref(), &mut factory);

    let mut tile_map_render_data: Vec<PlaneRenderer<_>> = Vec::new();
    let mut target_areas: HashMap<usize, Vec<usize>> = HashMap::new();
    for layer in map.layers.iter() {
        if layer.name == "meta" {
            for (y, cols) in layer.tiles.iter().enumerate() {
                for (x, cell) in cols.iter().enumerate() {
                    if *cell != 0 {
                        if !target_areas.contains_key(&x) {
                            target_areas.insert(x, vec![y]);
                        } else {
                            let mut ys = target_areas.get_mut(&x).unwrap();
                            ys.push(y);
                        }
                    }
                }
            }
        } else {
            let tilemap_plane = TileMapPlane::new(&map, &layer);
            tile_map_render_data.push(PlaneRenderer::new(&mut factory, &tilemap_plane, &tiles_texture, &target));
        }
    }

    let mut planner = {
        let mut world = World::new();
        world.add_resource::<Camera>(Camera(renderer::get_ortho()));
        world.add_resource::<Input>(Input::new(window.hidpi_factor(), vec![VirtualKeyCode::W, VirtualKeyCode::A, VirtualKeyCode::S, VirtualKeyCode::D]));
        world.add_resource::<TileData>(TileData::new(target_areas, &map));
        world.register::<HighlightTile>();
        world.register::<Sprite>();
        world.register::<Transform>();
        world.register::<Player>();
        world.create_now().with(Transform::new(0, 64, 32, 64, 0.0, 1.0, 1.0)).with(Sprite{ frame_name: String::from("player.png"), visible: true }).with(Player{});
        world.create_now().with(Transform::new(0, 0, 32, 32, 0.0, 1.0, 1.0)).with(Sprite{ frame_name: String::from("transparenttile.png"), visible: false }).with(HighlightTile{});
        Planner::<()>::new(world)
    };

    planner.add_system(systems::PlayerMovement{}, "player_movement", 1);

    let asset_data = loader::read_text_from_file("./resources/assets.json").unwrap();
    let spritesheet: Spritesheet = serde_json::from_str(asset_data.as_ref()).unwrap();
    let asset_texture = loader::gfx_load_texture("./resources/assets.png", &mut factory);

    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::MouseMoved(x, y) => {
                    let world = planner.mut_world();
                    let mut input = world.write_resource::<Input>().wait();
                    input.mouse_pos.0 = (x as f32 / input.hidpi_factor) as i32;
                    input.mouse_pos.1 = (y as f32 / input.hidpi_factor) as i32;
                },
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

        planner.dispatch(());

        basic.reset_transform();

        encoder.clear(&target.color, [16.0 / 256.0, 14.0 / 256.0, 22.0 / 256.0, 1.0]);
        encoder.clear_depth(&target.depth, 1.0);

        for plane_renderer in tile_map_render_data.iter_mut() {
            plane_renderer.render(&mut encoder, planner.mut_world());
        }

        let world = planner.mut_world();
        let sprites = world.read::<Sprite>().pass();
        let transforms = world.read::<Transform>().pass();

        for (sprite, transform) in (&sprites, &transforms).join() {
            if sprite.visible {
                basic.render(&mut encoder, world, &mut factory, &transform, &sprite, &spritesheet, &asset_texture);
            }
        }

        encoder.flush(&mut device);

        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
