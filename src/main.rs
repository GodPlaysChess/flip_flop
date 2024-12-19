mod game_entities;

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use minifb::Key::W;
use log::log;
use crate::game_entities::{Board, BOARD_SIZE, Cell, CELL_SIZE, GameState, Shape, ShapeType};

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;

const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const WHITE: u32 = 0xffffff;
const BLACK: u32 = 0x000000;

fn main() {
    let mut window = Window::new(
        "Board Game",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
        .unwrap();

    let mut buffer = vec![0; WIDTH * HEIGHT]; // Pixel buffer

    // Initialize the board and shapes
    let mut game: GameState = GameState::new();
    game.board.set_cell(3, 3, Cell::Filled); // Fill some cells for testing
    game.board.set_cell(4, 4, Cell::Filled);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(BLACK);
        handle_mouse(&mut game, &window);
        draw_game(&game, &mut buffer);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn draw_game(game_state: &GameState, buffer: &mut Vec<u32>) {
    draw_board(game_state, buffer);
    let mut i = 0;
    for shape in &game_state.shape_choice {
        draw_shape(shape, RED, buffer);
        i += 1;
    }

    draw_score(game_state.score, buffer);
    draw_cursor(&game_state.selected_shape, game_state.mouse_position, buffer);
    draw_mouse_click(game_state.last_click_position, buffer);
}

fn draw_cursor(selected_shape: &Option<ShapeType>, mouse_position: (usize, usize), buffer: &mut Vec<u32>) {
    if let Some(kind) = selected_shape {
        let (mx, my) = mouse_position;
        draw_shape_kind(kind, mx, my, GREEN, buffer);
    }
}

fn draw_mouse_click(x_y: (usize, usize), buffer: &mut Vec<u32>) {
    for i in 0..10 {
        for j in 0..10 {
            let y = x_y.1 + i;
            let x = x_y.0 + j;
            buffer[(y * WIDTH + x) % (WIDTH * HEIGHT)] = GREEN
        }
    }
}

fn draw_score(score: u32, buffer: &mut Vec<u32>) {
    let text = format!("Score: {}", score);
    let start_x = WIDTH - text.len() * 8 * 2; // Adjust for text length
    let start_y = 10;

    // Simplified text rendering using rectangles for each "pixel"
    for (i, c) in text.chars().enumerate() {
        let char_x = start_x + i * 16; // 16 pixels per character
        draw_char(c, char_x, start_y, buffer);
    }
}

fn draw_char(c: char, x: usize, y: usize, buffer: &mut Vec<u32>) {
    let font_data = get_font_data(c); // You can define your font data for ASCII chars
    for (row, line) in font_data.iter().enumerate() {
        for (col, &pixel) in line.iter().enumerate() {
            if pixel == 1 {
                let px = x + col;
                let py = y + row;
                if px < WIDTH && py < HEIGHT {
                    buffer[py * WIDTH + px] = WHITE; // White color for text
                }
            }
        }
    }
}

fn get_font_data(c: char) -> Vec<Vec<u8>> {
    // Define font data (simplified for this example)
    match c {
        '0' => vec![
            vec![0, 1, 1, 1, 0],
            vec![1, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 1],
            vec![0, 1, 1, 1, 0],
        ],
        '1' => vec![
            vec![0, 0, 1, 0, 0],
            vec![0, 1, 1, 0, 0],
            vec![0, 0, 1, 0, 0],
            vec![0, 1, 1, 1, 0],
        ],
        // Add definitions for other characters
        _ => vec![vec![0; 5]; 5], // Blank for undefined characters
    }
}

fn draw_board(game: &GameState, buffer: &mut Vec<u32>) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            let color = match game.board.grid[y][x] {
                Cell::Empty => 0x202020, // Dark gray for empty cells
                Cell::Filled => WHITE, // White for filled cells
            };
            draw_cell(x, y, color, buffer);
        }
    }

    // Highlight placement zone if a shape is selected
    if let Some(kind) = game.selected_shape {
        let (x, y) = game.mouse_position;
        let n = x / CELL_SIZE;
        let m = y / CELL_SIZE;

        let valid = game.is_valid_placement(&kind, n, m);
        let color = if valid { GREEN } else { RED };
        for (dx, dy) in Shape::cells(&kind)  {
            let nx = n.wrapping_add(dx);
            let ny = m.wrapping_add(dy);
            if nx < BOARD_SIZE && ny < BOARD_SIZE {
                draw_cell(nx, ny, color, buffer);
            }
        }
    }
}

fn draw_shape(shape: &Shape, color: u32, buffer: &mut Vec<u32>) {
    for (x, y) in &shape.cells {
        let px = shape.bot_left_pos.0 + x * CELL_SIZE;
        let py = shape.bot_left_pos.1 + y * CELL_SIZE;
        draw_rect(px, py, CELL_SIZE, CELL_SIZE, color, buffer);
    }
}

fn draw_shape_kind(shape: &ShapeType, pos_x: usize, pos_y: usize, color: u32, buffer: &mut Vec<u32>) {
    for (x, y) in Shape::cells(shape) {
        let px = pos_x + x * CELL_SIZE;
        let py = pos_y + y * CELL_SIZE;
        draw_rect(px, py, CELL_SIZE, CELL_SIZE, color, buffer);
    }
}

fn draw_cell(n: usize, m: usize, color: u32, buffer: &mut Vec<u32>) {
    let px = n * CELL_SIZE;
    let py = m * CELL_SIZE;
    draw_rect(px, py, CELL_SIZE, CELL_SIZE, color, buffer);
}

fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: u32, buffer: &mut Vec<u32>) {
    for row in 0..height {
        for col in 0..width {
            let px = x + col;
            let py = y + row;
            if px < WIDTH && py < HEIGHT {
                buffer[py * WIDTH + px] = color;
            }
        }
    }
}

fn handle_mouse(game: &mut GameState, window: &Window) {
    // Get the mouse position in board coordinates
    if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
        let cell_x = (mx as usize) / CELL_SIZE;
        let cell_y = (my as usize) / CELL_SIZE;
        game.mouse_position = (mx as usize, my as usize);
    }

    // Check for left click to select a shape
    if window.get_mouse_down(MouseButton::Left) {
        println!("Mouse clicked: {} : {}", game.mouse_position.0, game.mouse_position.1);
        game.last_click_position = game.mouse_position.clone();
        for shape in game.shape_choice.iter() {
            if is_mouse_over_shape(game.mouse_position, shape.bot_left_pos.0, shape.bot_left_pos.1) {
                game.selected_shape = Some(shape.kind.clone());
                break;
            }
        }
    }

    // Check for right click to deselect
    if window.get_mouse_down(MouseButton::Right) {
        game.selected_shape = None;
    }
}

fn is_mouse_over_shape(mouse: (usize, usize), shape_x: usize, shape_y: usize) -> bool {
    // todo proper collision detection with polygon
    let (mx, my) = mouse;
    let inbound = mx >= shape_x && mx < (shape_x + CELL_SIZE) && my >= shape_y && my < (shape_y + CELL_SIZE);
    println!("Selected: {}", inbound);
    return inbound;
}
