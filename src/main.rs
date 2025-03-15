use winit::{event::*, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder};
use winit::event_loop::EventLoopWindowTarget;

use render::render::Render;

use crate::events::Event::{Resize, ScoreUpdated};
use crate::game_entities::{Cell, GameState};
use crate::input::Input;
use crate::render::render::UserRenderConfig;

mod game_entities;
mod events;
mod render;
mod logic;
mod system;
mod input;
mod sound;


pub async fn run() {
    let config = UserRenderConfig::default();

    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let size = config.window_size;
    let window = WindowBuilder::new()
        .with_visible(false)
        .with_title("flip flop")
        .with_inner_size(size)
        .build(&event_loop).unwrap();

    // todo will change it to better version of cursor
    // window.set_cursor_visible(false);

    let mut render = pollster::block_on(Render::new(&window, config));
    let mut game = GameState::new();
    game.board.set_cell(4, 3, Cell::Filled);
    game.board.set_cell(7, 1, Cell::Filled);
    game.board.set_cell(0, 0, Cell::Filled);


    let sound_system = sound::SoundSystem::new();
    let sound_pack = sound::SoundPack::new();
    let mut game_event_queue: Vec<events::Event> = Vec::new();
    let mut input = Input::new();

    // todo initialise all systems
    // systems would handle inputs

    window.set_visible(true);
    let mut last_time = instant::Instant::now();


    // logic::handle_input(&mut game, &window, &mut game_event_queue);
    // logic::game_loop(&mut game, &mut game_event_queue);

    let window = &window;
    let mut cursor_position = (0.0, 0.0);
    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        state: element_state,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                    ..
                },
                ..
            } => {
                let input_handled = input.update(&key, &element_state);
                if !input_handled {
                    ignore_input(&element_state, &key, control_flow);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved {
                    position,
                    ..
                }, ..
            } => {
                cursor_position = (position.x, position.y);
                // println!("Cursor moved to: {:?}", cursor_position);
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested, ..
            } => {
                last_time = instant::Instant::now();
                window.request_redraw();

                for event in &game_event_queue {
                    match event {
                        events::Event::FocusChanged | events::Event::ButtonPressed => {
                            sound_system.queue(sound_pack.bounce());
                        }
                        ScoreUpdated(u32) => {
                            sound_system.queue(sound_pack.bounce());
                        }
                        events::Event::BoardUpdated(_) => {
                            //todo logic
                        }
                        events::Event::ShapeChoiceUpdate => {
                            // todo logic
                        }
                        Resize(_, _) => {}
                    }
                }
                game_event_queue.clear();
                render.render_state(&game);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                    ..
                }, ..
            } => control_flow.exit(),

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                render.resize(size);
                game_event_queue.push(events::Event::Resize(size.width as f32, size.height as f32));
            }

            _ => {}
        }
    }).unwrap();
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


