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
extern crate linked_hash_map;

use std::collections::HashMap;
use std::ops::{DerefMut};

use gfx::Device;
use specs::{DispatcherBuilder, Join, World};

use std::path::Path;
use std::fs::File;
use tiled::parse;
use glutin::{ElementState, MouseButton, VirtualKeyCode};
use cgmath::Vector2;

mod renderer;
mod loader;
mod components;
mod math;
mod spritesheet;
mod systems;
mod utils;

use components::{Camera, HighlightTile, Input, Player, Sprite, TileData, Transform};

use renderer::{ColorFormat, DepthFormat};

use spritesheet::Spritesheet;

fn setup_world(world: &mut World, window: &glutin::Window, walkable_groups: Vec<HashMap<usize, Vec<usize>>>, map: &tiled::Map) {
    world.add_resource::<Camera>(Camera(renderer::get_ortho()));
    world.add_resource::<Input>(Input::new(window.hidpi_factor(), vec![VirtualKeyCode::W, VirtualKeyCode::A, VirtualKeyCode::S, VirtualKeyCode::D]));
    world.add_resource::<TileData>(TileData::new(walkable_groups, map));
    world.register::<HighlightTile>();
    world.register::<Sprite>();
    world.register::<Transform>();
    world.register::<Player>();

    let player_pos = Vector2::new(0, 64);

    world.create_entity().with(Transform::new(player_pos.x, player_pos.y, 32, 64, 0.0, 1.0, 1.0)).with(Sprite{ frame_name: String::from("player.png"), visible: true }).with(Player::new());
    world.create_entity().with(Transform::new(0, 0, 32, 32, 0.0, 1.0, 1.0)).with(Sprite{ frame_name: String::from("transparenttile.png"), visible: false }).with(HighlightTile{});

    let mut tile_data_res = world.write_resource::<TileData>();
    let mut tile_data = tile_data_res.deref_mut();
    if !tile_data.set_player_group_index_from_pos(&player_pos) {
        println!("Start position not on ground: {:?}", player_pos);
    }
}

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

    let mut tile_map_render_data = utils::tiled::get_map_render_data(&map, &tiles_texture, &mut factory, &target);
    let (walkable_groups, unpassable_tiles) = utils::tiled::parse_out_map_layers(&map);

    let mut world = World::new();
    setup_world(&mut world, &window, walkable_groups, &map);

    let pathable_grid: Vec<Vec<math::astar::TileType>> = math::astar::build_grid_for_map(&unpassable_tiles, map.width as usize, map.height as usize);

    let mut dispatcher = DispatcherBuilder::new()
        .add(systems::PlayerMovement{ pathable_grid: pathable_grid }, "player_movement", &[])
        .build();

    let asset_data = loader::read_text_from_file("./resources/assets.json").unwrap();
    let spritesheet: Spritesheet = serde_json::from_str(asset_data.as_ref()).unwrap();
    let asset_texture = loader::gfx_load_texture("./resources/assets.png", &mut factory);

    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::MouseMoved(x, y) => {
                    let mut input_res = world.write_resource::<Input>();
                    let mut input = input_res.deref_mut();
                    input.mouse_pos.0 = (x as f32 / input.hidpi_factor) as i32;
                    input.mouse_pos.1 = (y as f32 / input.hidpi_factor) as i32;
                },
                glutin::Event::MouseInput(mouse_state, MouseButton::Left) => {
                    let mut input_res = world.write_resource::<Input>();
                    let mut input = input_res.deref_mut();
                    match mouse_state {
                        ElementState::Pressed => input.mouse_pressed = true,
                        ElementState::Released => input.mouse_pressed = false,
                    };
                },
                glutin::Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) | glutin::Event::Closed => break 'main,
                glutin::Event::KeyboardInput(key_state, _, key) => {
                    let mut input_res = world.write_resource::<Input>();
                    let mut input = input_res.deref_mut();
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

        dispatcher.dispatch(&mut world.res);

        basic.reset_transform();

        encoder.clear(&target.color, [16.0 / 256.0, 14.0 / 256.0, 22.0 / 256.0, 1.0]);
        encoder.clear_depth(&target.depth, 1.0);

        for plane_renderer in tile_map_render_data.iter_mut() {
            plane_renderer.render(&mut encoder, &mut world);
        }

        let sprites = world.read::<Sprite>();
        let transforms = world.read::<Transform>();

        for (sprite, transform) in (&sprites, &transforms).join() {
            if sprite.visible {
                basic.render(&mut encoder, &world, &mut factory, &transform, &sprite, &spritesheet, &asset_texture);
            }
        }

        encoder.flush(&mut device);

        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
