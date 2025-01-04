mod game_entities;
mod events;
mod render;
mod logic;

use std::collections::VecDeque;
use minifb::{Key, Window, WindowOptions};
use render::renderer::BLACK;
use crate::game_entities::{Cell, GameState};
use crate::render::renderer::Renderer;
use wgpu::*;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};
use render::state::State;
use render::renderer::*;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;

// settings
const TARGET_FPS: f32 = 240.0;
const N_SHAPES_PER_TURN: usize = 3;

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = State::new(&window).await;
    let mut surface_configured = false;

    event_loop.run(move |event, control_flow| {
        let mut cursor_position = (0.0, 0.0);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = (position.x, position.y);
                        println!("Cursor moved to: {:?}", cursor_position);
                    }
                    WindowEvent::RedrawRequested => {
                        if !surface_configured {
                            return;
                        }
                        // This tells winit that we want another frame after this one
                        state.window().request_redraw();
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if it's lost or outdated
                            Err(
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                            ) => state.resize(state.size),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                log::error!("OutOfMemory");
                                control_flow.exit();
                            }

                            // This happens when the a frame takes too long to present
                            Err(wgpu::SurfaceError::Timeout) => {
                                log::warn!("Surface timeout")
                            }
                        }
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        surface_configured = true;
                        state.resize(*physical_size);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }).unwrap();
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


