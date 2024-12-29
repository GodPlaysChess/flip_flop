use std::collections::VecDeque;
use minifb::{Key, Window, WindowOptions};
use render::BLACK;
use crate::events::Event;
use crate::game_entities::{Cell, GameState};
use crate::render::Renderer;

mod game_entities;
mod events;
mod render;
mod logic;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;

// settings
const TARGET_FPS: f32 = 240.0;
const N_SHAPES_PER_TURN: usize = 3;

fn main() {
    let mut window = Window::new(
        "Board Game",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap();

    // board, score, shape choice
    let mut background_buffer = vec![0u32; WIDTH * HEIGHT]; // Static elements
    // mouse, highlights
    let mut foreground_buffer = vec![0u32; WIDTH * HEIGHT]; // Dynamic elements
    let mut rendered_buffer = vec![0u32; WIDTH * HEIGHT];   // Final output

    // Initialize the board and shapes
    let mut game: GameState = GameState::new();
    game.board.set_cell(3, 3, Cell::Filled); // Fill some cells for testing
    game.board.set_cell(4, 4, Cell::Filled);
    let frame_duration = 1.0 / TARGET_FPS;


    // Load the font file (ensure "DejaVuSans.ttf" is in your project directory)
    let font_data = include_bytes!("resources\\DejaVuSans.ttf");
    let mut renderer = Renderer { width: WIDTH, height: HEIGHT };


    // draw initial screen
    background_buffer.fill(BLACK);
    render::draw_background(&game, &mut background_buffer, font_data, &mut renderer);

    let mut event_queue: VecDeque<Event> = VecDeque::new();
    let mut last_time = std::time::Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last_time).as_secs_f32();
        if elapsed < frame_duration {
            std::thread::sleep(std::time::Duration::from_secs_f32(frame_duration - elapsed));
        }

        logic::handle_input(&mut game, &window, &mut event_queue);

        logic::game_loop(&mut game, &mut event_queue);

        foreground_buffer.fill(0);
        while !event_queue.is_empty() {
            if let Some(event) = event_queue.pop_front() {
                render::update_background(event, &game, &mut background_buffer, &mut renderer, font_data);
            }
        }

        render::draw_foreground(&game, &mut foreground_buffer);

        // Combine buffers
        for i in 0..(WIDTH * HEIGHT) {
            rendered_buffer[i] = if foreground_buffer[i] != 0 {
                foreground_buffer[i]
            } else {
                background_buffer[i]
            };
        }

        window.update_with_buffer(&rendered_buffer, WIDTH, HEIGHT).unwrap();
        last_time = now;
    }
}


