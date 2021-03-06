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
use specs::{Dispatcher, DispatcherBuilder, Join, World};

use std::path::Path;
use std::fs::File;
use tiled::parse;
use glutin::{Event, ElementState, MouseButton, VirtualKeyCode, WindowEvent};
use glutin::GlContext;
use cgmath::Vector2;

mod renderer;
mod loader;
mod components;
mod math;
mod spritesheet;
mod systems;
mod utils;
mod types;
use types::TileMapping;

use components::{AnimationSheet, Camera, Color, Enemy, EnergyBar, GameState, HighlightTile, Input, Player, Rect, Sprite, TileData, Transform};

use renderer::{ColorFormat, DepthFormat};

use spritesheet::Spritesheet;

fn setup_world<'a>(world: &mut World, window: &glutin::Window, walkable_groups: Vec<TileMapping<usize>>, map: &tiled::Map, jump_targets: TileMapping<usize>, pathable_grid: Vec<Vec<math::astar::TileType>>) -> Dispatcher<'a, 'a> {
    world.add_resource::<Camera>(Camera(renderer::get_ortho()));
    world.add_resource::<Input>(Input::new(window.hidpi_factor(), vec![VirtualKeyCode::W, VirtualKeyCode::A, VirtualKeyCode::S, VirtualKeyCode::D]));
    world.add_resource::<TileData>(TileData::new(walkable_groups, map, jump_targets));
    world.add_resource::<GameState>(GameState::new());
    world.register::<AnimationSheet>();
    world.register::<Color>();
    world.register::<Enemy>();
    world.register::<EnergyBar>();
    world.register::<HighlightTile>();
    world.register::<Rect>();
    world.register::<Sprite>();
    world.register::<Transform>();
    world.register::<Player>();

    let player_pos = Vector2::new(0, 64);

    let player_entity = world.create_entity()
        .with(Transform::new(player_pos.x, player_pos.y, 32, 64, 0.0, 1.0, 1.0))
        .with(Sprite{ frame_name: String::from("player.png"), visible: true })
        .with(Player::new())
        .build();

    world.create_entity()
        .with(Transform::new(800, 610, EnergyBar::get_max_width(), 25, 0.0, 1.0, 1.0))
        .with(EnergyBar{})
        .with(Rect{})
        .with(Color([0.0, 1.0, 0.0, 1.0]));

    let mut animation_sheet = AnimationSheet::new(0.1);
    animation_sheet.add_animation(String::from("idle"), vec![
        String::from("skeleton_1.png"),
        String::from("skeleton_2.png"),
        String::from("skeleton_3.png"),
        String::from("skeleton_4.png"),
    ]);
    animation_sheet.set_current_animation(String::from("idle"));
    world.create_entity()
        .with(Transform::new(128, 160, 32, 32, 0.0, 1.0, 1.0))
        .with(animation_sheet)
        .with(Enemy{});
    world.create_entity()
        .with(Transform::new(0, 0, 32, 32, 0.0, 1.0, 1.0))
        .with(Sprite{ frame_name: String::from("transparenttile.png"), visible: false })
        .with(HighlightTile{});

    let mut tile_data_res = world.write_resource::<TileData>();
    let mut tile_data = tile_data_res.deref_mut();
    if !tile_data.set_player_group_index_from_pos(&player_pos) {
        println!("Start position not on ground: {:?}", player_pos);
    }

    DispatcherBuilder::new()
        .add(systems::PlayerMovement{ pathable_grid: pathable_grid }, "player_movement", &[])
        .add(systems::ProcessTurn{}, "process_turn", &[])
        .add(systems::AnimationSystem::new(), "animation_system", &[])
        .add(systems::EnergyUi{ player_entity: player_entity }, "energy_ui", &["player_movement"])
        .build()
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let dim = renderer::get_dimensions();
    let builder = glutin::WindowBuilder::new()
        .with_title("Agile Archer".to_string())
        .with_dimensions(dim[0] as u32, dim[1] as u32);

    let context = glutin::ContextBuilder::new();

    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop);

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
    let (walkable_groups, jump_targets, unpassable_tiles) = utils::tiled::parse_out_map_layers(&map);

    let mut world = World::new();
    let pathable_grid: Vec<Vec<math::astar::TileType>> = math::astar::build_grid_for_map(&unpassable_tiles, map.width as usize, map.height as usize);
    let mut dispatcher = setup_world(&mut world, &window, walkable_groups, &map, jump_targets, pathable_grid);

    let asset_data = loader::read_text_from_file("./resources/assets.json").unwrap();
    let spritesheet: Spritesheet = serde_json::from_str(asset_data.as_ref()).unwrap();
    let asset_texture = loader::gfx_load_texture("./resources/assets.png", &mut factory);

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent{ event, .. } => match event {
                    WindowEvent::MouseMoved{ position: (x, y), .. } => {
                        let mut input_res = world.write_resource::<Input>();
                        let mut input = input_res.deref_mut();
                        input.mouse_pos.0 = (x as f32 / input.hidpi_factor) as i32;
                        input.mouse_pos.1 = (y as f32 / input.hidpi_factor) as i32;
                    },
                    WindowEvent::MouseInput{ button: MouseButton::Left, state, .. } => {
                        let mut input_res = world.write_resource::<Input>();
                        let mut input = input_res.deref_mut();
                        match state {
                            ElementState::Pressed => input.mouse_pressed = true,
                            ElementState::Released => input.mouse_pressed = false,
                        };
                    },
                    WindowEvent::KeyboardInput{ input: glutin::KeyboardInput{ virtual_keycode: Some(VirtualKeyCode::Escape), .. }, .. } | glutin::WindowEvent::Closed => running = false,
                    WindowEvent::KeyboardInput{ input, .. } => {
                        let input_event = input;
                        let mut input_res = world.write_resource::<Input>();
                        let mut input = input_res.deref_mut();
                        if let Some(key) = input_event.virtual_keycode {
                            if input.pressed_keys.contains_key(&key) {
                                match input_event.state {
                                    ElementState::Pressed => input.pressed_keys.insert(key, true),
                                    ElementState::Released => input.pressed_keys.insert(key, false),
                                };
                            }
                        }
                    },
                    _ => {}
                },
                _ => ()
            }
        });

        dispatcher.dispatch(&mut world.res);

        basic.reset_transform();

        encoder.clear(&target.color, [16.0 / 256.0, 14.0 / 256.0, 22.0 / 256.0, 1.0]);
        encoder.clear_depth(&target.depth, 1.0);

        for plane_renderer in tile_map_render_data.iter_mut() {
            plane_renderer.render(&mut encoder, &mut world);
        }

        let sprites = world.read::<Sprite>();
        let transforms = world.read::<Transform>();
        let animation_sheets = world.read::<AnimationSheet>();
        let colors = world.read::<Color>();
        let rects = world.read::<Rect>();

        for (sprite, transform) in (&sprites, &transforms).join() {
            if sprite.visible {
                basic.render(&mut encoder, &world, &mut factory, &transform, Some(&sprite.frame_name), &spritesheet, None, Some(&asset_texture));
            }
        }

        for (animation_sheet, transform) in (&animation_sheets, &transforms).join() {
            basic.render(&mut encoder, &world, &mut factory, &transform, Some(animation_sheet.get_current_frame()), &spritesheet, None, Some(&asset_texture));
        }

        for (color, transform, _) in (&colors, &transforms, &rects).join() {
            basic.render(&mut encoder, &world, &mut factory, &transform, None, &spritesheet, Some(color.0), None);
        }

        encoder.flush(&mut device);

        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
