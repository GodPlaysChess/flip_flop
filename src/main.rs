use std::collections::{HashMap, HashSet, VecDeque};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use crate::events::{BoardUpdate, CellCoord, Event, XY};
use crate::events::Event::{BoardUpdated, ShapeChoiceUpdate};
use crate::game_entities::{BOARD_SIZE, Cell, CELL_SIZE, GameState, Shape, ShapeState, ShapeType};
use crate::game_entities::ShapeState::VISIBLE;
use crate::render::Renderer;

mod game_entities;
mod events;
mod render;

const WIDTH: usize = 1200;
const HEIGHT: usize = 800;

const SHAPE_LINE_COORD_X: usize = 100;
const SHAPE_LINE_COORD_Y: usize = 500;
const SPACE_BETWEEN_CELLS: usize = 5;

const WIDTH_CELLS: usize = (WIDTH - SHAPE_LINE_COORD_X) / CELL_SIZE;
const HEIGHT_CELLS: usize = (HEIGHT - SHAPE_LINE_COORD_Y) / CELL_SIZE;


const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const WHITE: u32 = 0xffffff;
const BLACK: u32 = 0x000000;

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
    draw_background(&game, &mut background_buffer, font_data, &mut renderer);

    let mut event_queue: VecDeque<Event> = VecDeque::new();
    let mut last_time = std::time::Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last_time).as_secs_f32();
        if elapsed < frame_duration {
            std::thread::sleep(std::time::Duration::from_secs_f32(frame_duration - elapsed));
        }

        handle_input(&mut game, &window, &mut event_queue);

        game_loop(&mut game, &mut event_queue);

        foreground_buffer.fill(0);
        while !event_queue.is_empty() {
            if let Some(event) = event_queue.pop_front() {
                update_background(event, &game, &mut background_buffer, &mut renderer, font_data);
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
        last_time = now;
    }
}


//After placing a shape, only check rows and columns affected by the shape.
//Use a bitmask for each row and column to track filled cells more efficiently.
fn game_loop(game_state: &mut GameState, event_queue: &mut VecDeque<Event>) {
    verify_that_row_is_full(game_state, event_queue);
    generate_new_shapes(game_state, event_queue);
}

fn generate_new_shapes(game_state: &mut GameState, event_queue: &mut VecDeque<Event>) {
    if game_state.selected_shape.is_none() && !game_state.shape_choice.iter().any(|s| s.state == VISIBLE) {
        game_state.shape_choice = Shape::get_random_choice(N_SHAPES_PER_TURN);
        event_queue.push_front(ShapeChoiceUpdate);
    }
}

fn verify_that_row_is_full(game_state: &mut GameState, event_queue: &mut VecDeque<Event>) {
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

    if upd_coord.is_empty() {
        return;
    }

    // filter update events that updates this row or column
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

fn update_background(event: Event, game_state: &GameState, buffer: &mut Vec<u32>, renderer: &mut Renderer, font_data: &[u8]) {
    match event {
        Event::ScoreUpdated(new_score) => {
            draw_score(new_score, renderer, font_data, buffer)
        }
        BoardUpdated(updates) => {
            for update in updates {
                draw_cell(update.coord.0, update.coord.1, &update.cell, buffer);
            }
        }
        // to simplify the deselecting etc, we just redraw the shapes below
        ShapeChoiceUpdate => {
            // println!("Updating background shape choice with shapes {:?}", game_state.shape_choice);
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

fn draw_background(game_state: &GameState, buffer: &mut Vec<u32>, font_data: &[u8], renderer: &mut Renderer) {
    // if board is changed
    draw_board(game_state, buffer);
    draw_shape_choice(game_state, buffer);
    // if score is changed
    draw_score(game_state.score, renderer, font_data, buffer);
}

fn draw_shape_choice(game_state: &GameState, buffer: &mut Vec<u32>) {
    // todo hardcoded coordinates and width
    draw_rect(SHAPE_LINE_COORD_X, SHAPE_LINE_COORD_Y, 1000, 300, BLACK, buffer);
    let mut pos_x = SHAPE_LINE_COORD_X;
    for shape in &game_state.shape_choice {
        pos_x += CELL_SIZE * shape.x_cell_coordinate;
        if (shape.state == VISIBLE) {
            draw_shape_kind(&shape.kind, pos_x, SHAPE_LINE_COORD_Y, RED, buffer);
        }
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

fn draw_score(score: u32, renderer: &Renderer, font_data: &[u8], buffer: &mut Vec<u32>) {
    renderer.draw_text(score.to_string().as_str(), 800, 20, font_data, 36.0, WHITE, buffer);
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

        let selected_shape = is_mouse_over_shape(game.mouse_position, &game.shape_choice);
        if let Some(i) = selected_shape {
            // place back previously selected shape
            game.deselect();

            let shape = game.shape_choice.get(i).unwrap().kind.clone();
            game.shape_choice.get_mut(i).unwrap().set_state(ShapeState::SELECTED);
            game.selected_shape = Some(shape);
            events.push_front(ShapeChoiceUpdate);
        }
    }

    // Check for right click to deselect
    if window.get_mouse_down(MouseButton::Right) {
        if game.selected_shape.is_some() {
            game.deselect();
            events.push_front(ShapeChoiceUpdate);
        }
    }
}


fn is_mouse_over_shape(mouse: (usize, usize), shapes: &Vec<Shape>) -> Option<usize> {
    let (mx, my) = mouse;
    // Transform mouse coordinates to grid space
    if mx < SHAPE_LINE_COORD_X || my < SHAPE_LINE_COORD_Y {
        return None;
    }

    let relative_x = mx - SHAPE_LINE_COORD_X;
    let relative_y = my - SHAPE_LINE_COORD_Y;
    // converting to the grid space
    let (col, row) = (relative_x / CELL_SIZE, relative_y / CELL_SIZE);
    let shapes_grid = as_grid(shapes);
    let ix = rc_to_ix(row, col);

    return shapes_grid.get(&ix)
        .cloned()
        .filter(|i| shapes.get(*i).is_some_and(|s| s.state == VISIBLE));
}

// converts shape choice to grid-like representation
fn as_grid(shapes: &Vec<Shape>) -> HashMap<usize, usize> {
    let mut result = HashMap::new();
    for (i, s) in shapes.iter().enumerate() {
        for (dx, dy) in Shape::cells(&s.kind) {
            let n = rc_to_ix(dy, dx + s.x_cell_coordinate);
            result.insert(n, i);
        }
    }

    return result;
}

// converts row/col coordinate to the cell index
fn rc_to_ix(r: usize, c: usize) -> usize {
    return WIDTH_CELLS * r + c;
}
