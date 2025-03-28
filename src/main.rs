use std::collections::VecDeque;
use winit::event_loop::EventLoopWindowTarget;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use render::render::Render;

use crate::events::Event::SelectedShapePlaced;
use crate::game_entities::{Cell, GameState, SelectedShape, ShapeState};
use crate::input::Input;
use crate::render::render::UserRenderConfig;
use crate::system::{PlacementSystem, ScoreCleanupSystem, SelectionValidationSystem, System};

mod events;
mod game_entities;
mod input;
mod render;
mod sound;
mod space_converters;
mod system;

pub async fn run() {
    let config = UserRenderConfig::default();

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let size = config.window_size;
    let window = WindowBuilder::new()
        .with_visible(false)
        .with_title("flip flop")
        .with_inner_size(size)
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    let mut render = pollster::block_on(Render::new(&window, config.clone()));
    let mut game = GameState::new(config.board_size_cols);
    game.board.set_cell(4, 3, Cell::Filled);
    game.board.set_cell(7, 1, Cell::Filled);
    game.board.set_cell(0, 0, Cell::Filled);

    let sound_system = sound::SoundSystem::new();
    let sound_pack = sound::SoundPack::new();
    let mut game_event_queue: VecDeque<events::Event> = VecDeque::new();
    let mut input = Input::new();

    let selection_system = SelectionValidationSystem;
    let placement_system = PlacementSystem;
    let score_cleanup_system = ScoreCleanupSystem;

    window.set_visible(true);
    let mut last_time = instant::Instant::now();

    let window = &window;
    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::WindowEvent {
                    event:
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        },
                    ..
                } => control_flow.exit(),
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: element_state,
                                    physical_key: PhysicalKey::Code(key),
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                    let input_handled = input.update_kb(&key, &element_state);
                    if !input_handled {
                        ignore_input(&element_state, &key, control_flow);
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    input.update_mouse_position(position);
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { button, state, .. },
                    ..
                } => {
                    input.update_mouse(&button, &state);
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let dt = last_time.elapsed();
                    last_time = instant::Instant::now();
                    window.request_redraw();

                    selection_system.update_state(
                        &input,
                        dt,
                        &mut game,
                        &mut game_event_queue,
                        &config,
                        None,
                    );

                    while let Some(event) = game_event_queue.pop_front() {
                        match event {
                            events::Event::ShapeSelected(n, coord) => {
                                game.deselect();
                                let selected_shape = game.panel.shape_choice.get_mut(n).unwrap();
                                game.selected_shape = Some(SelectedShape {
                                    shape_type: selected_shape.kind,
                                    anchor_offset: coord,
                                });
                                selected_shape.set_state(ShapeState::SELECTED);
                                println!("Shape {:?} is selected", &selected_shape);
                            }
                            SelectedShapePlaced(_, _) => {
                                placement_system.update_state(
                                    &input,
                                    dt,
                                    &mut game,
                                    &mut game_event_queue,
                                    &config,
                                    Some(&event),
                                );
                                score_cleanup_system.update_state(
                                    &input,
                                    dt,
                                    &mut game,
                                    &mut game_event_queue,
                                    &config,
                                    None,
                                );
                                sound_system.queue(sound_pack.bounce());
                            }
                        }
                    }

                    score_cleanup_system.update_state(
                        &input,
                        dt,
                        &mut game,
                        &mut game_event_queue,
                        &config,
                        None,
                    );

                    // todo pass UI instead of game?
                    render.render_state(&game, &input);
                    input.reset();
                    window.request_redraw();
                }

                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    render.resize(size);
                }

                _ => {}
            }
        })
        .unwrap();
}

fn ignore_input(
    element_state: &ElementState,
    keycode: &KeyCode,
    control_flow: &EventLoopWindowTarget<()>,
) {
    match (keycode, element_state) {
        (KeyCode::Escape, ElementState::Pressed) => control_flow.exit(),
        _ => {}
    }
}

fn main() {
    pollster::block_on(run());

    // let mut window = Window::new(
    //     "Board Game",
    //     WIDTH,
    //     HEIGHT,
    //     WindowOptions::default(),
    // ).unwrap();
    //
    // // board, score, shape choice
    // let mut background_buffer = vec![0u32; WIDTH * HEIGHT]; // Static elements
    // // mouse, highlights
    // let mut foreground_buffer = vec![0u32; WIDTH * HEIGHT]; // Dynamic elements
    // let mut rendered_buffer = vec![0u32; WIDTH * HEIGHT];   // Final output
    //
    // // Initialize the board and shapes
    // let mut game: GameState = GameState::new();
    // game.board.set_cell(3, 3, Cell::Filled); // Fill some cells for testing
    // game.board.set_cell(4, 4, Cell::Filled);
    // let frame_duration = 1.0 / TARGET_FPS;
    //
    //
    // // Load the font file (ensure "DejaVuSans.ttf" is in your project directory)
    // let font_data = include_bytes!("resources\\DejaVuSans.ttf");
    // let mut renderer = Renderer { width: WIDTH, height: HEIGHT };
    //
    //
    // // draw initial screen
    // background_buffer.fill(BLACK);
    // draw_background(&game, &mut background_buffer, font_data, &mut renderer);
    //
    // let mut event_queue: VecDeque<events::Event> = VecDeque::new();
    // let mut last_time = std::time::Instant::now();
    //
    // while window.is_open() && !window.is_key_down(Key::Escape) {
    //     let now = std::time::Instant::now();
    //     let elapsed = now.duration_since(last_time).as_secs_f32();
    //     if elapsed < frame_duration {
    //         std::thread::sleep(std::time::Duration::from_secs_f32(frame_duration - elapsed));
    //     }
    //
    //     logic::handle_input(&mut game, &window, &mut event_queue);
    //
    //     logic::game_loop(&mut game, &mut event_queue);
    //
    //     foreground_buffer.fill(0);
    //     while !event_queue.is_empty() {
    //         if let Some(event) = event_queue.pop_front() {
    //             update_background(event, &game, &mut background_buffer, &mut renderer, font_data);
    //         }
    //     }
    //
    //     draw_foreground(&game, &mut foreground_buffer);
    //
    //     // Combine buffers
    //     for i in 0..(WIDTH * HEIGHT) {
    //         rendered_buffer[i] = if foreground_buffer[i] != 0 {
    //             foreground_buffer[i]
    //         } else {
    //             background_buffer[i]
    //         };
    //     }
    //
    //     window.update_with_buffer(&rendered_buffer, WIDTH, HEIGHT).unwrap();
    //     last_time = now;
    // }
}
