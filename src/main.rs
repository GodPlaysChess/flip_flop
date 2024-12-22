use std::collections::{HashSet, VecDeque};

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};

use crate::events::{BoardUpdate, CellCoord, Event, XY};
use crate::events::Event::{BoardUpdated, ShapeChoiceUpdate};
use crate::game_entities::{BOARD_SIZE, Cell, CELL_SIZE, GameState, Shape, ShapeType};

mod game_entities;
mod events;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;


const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const WHITE: u32 = 0xffffff;
const BLACK: u32 = 0x000000;

const TARGET_FPS: f32 = 240.0;

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

    // draw initial screen
    background_buffer.fill(BLACK);
    draw_background(&game, &mut background_buffer);

    let mut event_queue: VecDeque<Event> = VecDeque::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut last_time = std::time::Instant::now();
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last_time).as_secs_f32();
        if elapsed < frame_duration {
            std::thread::sleep(std::time::Duration::from_secs_f32(frame_duration - elapsed));
        }
        last_time = now;
        handle_input(&mut game, &window, &mut event_queue);

        game_loop(&mut game, &mut event_queue);

        foreground_buffer.fill(0);
        while !event_queue.is_empty() {
            if let Some(event) = event_queue.pop_front() {
                update_background(event, &game, &mut background_buffer);
            }
        }

        draw_foreground(&game, &mut foreground_buffer);

        // Combine buffers
        for i in 0..(WIDTH * HEIGHT) {
            rendered_buffer[i] = if foreground_buffer[i] != 0 {
                foreground_buffer[i]
            } else {
                background_buffer[i]
            };
        }

        window.update_with_buffer(&rendered_buffer, WIDTH, HEIGHT).unwrap();
    }
}

//After placing a shape, only check rows and columns affected by the shape.
//Use a bitmask for each row and column to track filled cells more efficiently.
fn game_loop(game_state: &mut GameState, event_queue: &mut VecDeque<Event>) {
    // if row on the board is filled then score and remove it
    let mut filled_rows = Vec::new();
    let mut filled_cols = Vec::new();
    let mut upd_coord: HashSet<CellCoord> = HashSet::new();

    for r in 0..BOARD_SIZE {
        if game_state.board.grid[r].iter().all(|&cell| cell == Cell::Filled) {
            filled_rows.push(r);
            for c in 0..BOARD_SIZE {
                upd_coord.insert(CellCoord(c, r));
            }
        }
    }
    for c in 0..BOARD_SIZE {
        if (0..BOARD_SIZE).all(|r| game_state.board.grid[r][c] == Cell::Filled) {
            filled_cols.push(c);
            for r in 0..BOARD_SIZE {
                upd_coord.insert(CellCoord(c, r));
            }
        }
    }

    if (upd_coord.is_empty()) {
        return;
    }

    // filter update events that updates this row or column
    //event_queue.retain(|e| e)
    let mut updates = vec![];
    for event in event_queue.iter_mut() {
        if let BoardUpdated(ref mut updates) = event {
            updates.retain(|update| !upd_coord.contains(&update.coord));
        }
    }
    for r in upd_coord {
        game_state.board.grid[r.1][r.0] = Cell::Empty;
        updates.push(BoardUpdate { cell: Cell::Empty, coord: r });
    }
    event_queue.push_front(BoardUpdated(updates));
}

fn update_background(event: Event, game_state: &GameState, buffer: &mut Vec<u32>) {
    match event {
        Event::ScoreUpdated(new_score) => {
            draw_score(new_score, buffer)
        }
        BoardUpdated(updates) => {
            for update in updates {
                draw_cell(update.coord.0, update.coord.1, &update.cell, buffer);
            }
        }
        // to simplify the deselecting etc, we just redraw the shapes below
        ShapeChoiceUpdate => {
            println!("Updating background shape choice with shapes {:?}", game_state.shape_choice);
            draw_shape_choice(game_state, buffer);
        }
    }
}

fn draw_foreground(game_state: &GameState, buffer: &mut Vec<u32>) {
    draw_cursor(&game_state.selected_shape, game_state.mouse_position, buffer);
    if let Some(shape) = game_state.selected_shape {
        let valid_placement = game_state.is_valid_placement_of_selected_shape();
        draw_highlight(&shape, XY(game_state.mouse_position.0, game_state.mouse_position.1), valid_placement, buffer);
    }

    // if mouse is clicked
    draw_mouse_click(game_state.last_click_position, buffer);
}

// let's make an optimisation that only redraws the changing elements, instead of drawing the whole state every time.
// what's changing:
// 1. shapes after move being done
// 2. if shape is selected -> cursor become shape
// 3. when shape moves previous position is returned back to what it was, current mouse position got updated
// 4. when placed -> board is updated
fn draw_background(game_state: &GameState, buffer: &mut Vec<u32>) {
    // if board is changed
    draw_board(game_state, buffer);
    draw_shape_choice(game_state, buffer);
    // if score is changed
    draw_score(game_state.score, buffer);
}

fn draw_shape_choice(game_state: &GameState, buffer: &mut Vec<u32>) {
    // todo hardcoded coordinates and width
    draw_rect(100, 500, 500, 200, BLACK, buffer);
    for shape in &game_state.shape_choice {
        draw_shape(shape, RED, buffer);
    }
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
            draw_cell(x, y, &game.board.grid[y][x], buffer);
        }
    }
}

fn draw_highlight(kind: &ShapeType, mouse_position: XY, valid: bool, buffer: &mut Vec<u32>) {
    // Highlight placement zone if a shape is selected
    //todo duplication here and inside game object
    let XY(x, y) = mouse_position;
    let n = x / CELL_SIZE;
    let m = y / CELL_SIZE;

    let color = if valid { GREEN } else { RED };
    for (dx, dy) in Shape::cells(kind) {
        let nx = n.wrapping_add(dx);
        let ny = m.wrapping_add(dy);
        if nx < BOARD_SIZE && ny < BOARD_SIZE {
            draw_colored_cell(nx, ny, color, buffer);
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

fn draw_colored_cell(n: usize, m: usize, color: u32, buffer: &mut Vec<u32>) {
    let px = n * CELL_SIZE;
    let py = m * CELL_SIZE;
    draw_rect(px, py, CELL_SIZE, CELL_SIZE, color, buffer);
}

fn draw_cell(n: usize, m: usize, cell: &Cell, buffer: &mut Vec<u32>) {
    let color = match cell {
        Cell::Empty => 0x202020, // Dark gray for empty cells
        Cell::Filled => WHITE, // White for filled cells
    };
    draw_colored_cell(n, m, color, buffer);
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

fn handle_input(game: &mut GameState, window: &Window, events: &mut VecDeque<Event>) {
    // Get the mouse position in board coordinates
    if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
        game.mouse_position = (mx as usize, my as usize);
    }

    // Check for left click to select a shape
    if window.get_mouse_down(MouseButton::Left) {
        game.last_click_position = game.mouse_position.clone();

        if game.selected_shape.is_some() && game.is_valid_placement_of_selected_shape() {
            game.place_shape(events)
        }

        let mut i = 0;
        while i < game.shape_choice.len() {
            let (x, y) = game.shape_choice[i].bot_left_pos;
            if is_mouse_over_shape(game.mouse_position, x, y) {
                let shape = game.shape_choice.swap_remove(i);
                if let Some(currently_selected) = game.selected_shape {
                    game.shape_choice.push(Shape::new(currently_selected.to_owned(), (100, 500)));
                }
                game.selected_shape = Some(shape.kind.clone());
                events.push_front(ShapeChoiceUpdate);
                break;
            } else {
                i += 1; // Only increment if no removal happens
            }
        }
    }

    // Check for right click to deselect
    if window.get_mouse_down(MouseButton::Right) {
        if let Some(shape) = game.selected_shape {
            // todo change this data structure that just contains shape choice, not coordinates.
            game.shape_choice.push(Shape::new(shape, (300, 500)));
            events.push_front(ShapeChoiceUpdate);
        }
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
