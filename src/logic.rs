// use std::collections::{HashMap, HashSet, VecDeque};
// use minifb::{MouseButton, MouseMode, Window};
// use crate::events::{BoardUpdate, CellCoord, Event};
// use crate::game_entities::{BOARD_SIZE, Cell, CELL_SIZE, GameState, Shape, ShapeState};
// use crate::game_entities::ShapeState::VISIBLE;
//
// pub const SHAPE_LINE_COORD_X: usize = 100;
// pub const SHAPE_LINE_COORD_Y: usize = 500;
// const SPACE_BETWEEN_CELLS: usize = 5;
// const WIDTH_CELLS: usize = 40;
// // const HEIGHT_CELLS: usize = (SCREEN_HEIGHT - SHAPE_LINE_COORD_Y) / CELL_SIZE;
// // settings
// const N_SHAPES_PER_TURN: usize = 3;
//
// //After placing a shape, only check rows and columns affected by the shape.
// //Use a bitmask for each row and column to track filled cells more efficiently.
// pub fn game_loop(game_state: &mut GameState, event_queue: &mut VecDeque<Event>) {
//     verify_that_row_is_full(game_state, event_queue);
//     generate_new_shapes(game_state, event_queue);
// }
//
// fn generate_new_shapes(game_state: &mut GameState, event_queue: &mut VecDeque<Event>) {
//     if game_state.selected_shape.is_none() && !game_state.shape_choice.iter().any(|s| s.state == VISIBLE) {
//         game_state.shape_choice = Shape::get_random_choice(N_SHAPES_PER_TURN);
//         event_queue.push_front(ShapeChoiceUpdate);
//     }
// }
//
// fn verify_that_row_is_full(game_state: &mut GameState, event_queue: &mut VecDeque<Event>) {
//     // if row on the board is filled then score and remove it
//     let mut filled_rows = Vec::new();
//     let mut filled_cols = Vec::new();
//     let mut upd_coord: HashSet<CellCoord> = HashSet::new();
//
//     for r in 0..BOARD_SIZE {
//         if game_state.board.grid[r].iter().all(|&cell| cell == Cell::Filled) {
//             filled_rows.push(r);
//             for c in 0..BOARD_SIZE {
//                 upd_coord.insert(CellCoord(c, r));
//             }
//         }
//     }
//     for c in 0..BOARD_SIZE {
//         if (0..BOARD_SIZE).all(|r| game_state.board.grid[r][c] == Cell::Filled) {
//             filled_cols.push(c);
//             for r in 0..BOARD_SIZE {
//                 upd_coord.insert(CellCoord(c, r));
//             }
//         }
//     }
//
//     if upd_coord.is_empty() {
//         return;
//     }
//
//     // filter update events that updates this row or column
//     let mut updates = vec![];
//     for event in event_queue.iter_mut() {
//         if let BoardUpdated(ref mut updates) = event {
//             updates.retain(|update| !upd_coord.contains(&update.coord));
//         }
//     }
//     for r in upd_coord {
//         game_state.board.grid[r.1][r.0] = Cell::Empty;
//         updates.push(BoardUpdate { cell: Cell::Empty, coord: r });
//     }
//     game_state.score = game_state.score + updates.len() as u32;
//     event_queue.push_front(BoardUpdated(updates));
//
//     event_queue.push_front(ScoreUpdated(game_state.score));
// }
//
// pub fn handle_input(game: &mut GameState, window: &Window, events: &mut VecDeque<Event>) {
//     // Get the mouse position in board coordinates
//     if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
//         game.mouse_position = (mx as usize, my as usize);
//     }
//
//     // Check for left click to select a shape
//     if window.get_mouse_down(MouseButton::Left) {
//         game.last_click_position = game.mouse_position.clone();
//
//         if game.selected_shape.is_some() && game.is_valid_placement_of_selected_shape() {
//             game.place_shape(events)
//         }
//
//         let selected_shape = is_mouse_over_shape(game.mouse_position, &game.shape_choice);
//         if let Some(i) = selected_shape {
//             // place back previously selected shape
//             game.deselect();
//
//             let shape = game.shape_choice.get(i).unwrap().kind.clone();
//             game.shape_choice.get_mut(i).unwrap().set_state(ShapeState::SELECTED);
//             game.selected_shape = Some(shape);
//             events.push_front(ShapeChoiceUpdate);
//         }
//     }
//
//     // Check for right click to deselect
//     if window.get_mouse_down(MouseButton::Right) {
//         if game.selected_shape.is_some() {
//             game.deselect();
//             events.push_front(ShapeChoiceUpdate);
//         }
//     }
// }
//
// fn is_mouse_over_shape(mouse: (usize, usize), shapes: &Vec<Shape>) -> Option<usize> {
//     let (mx, my) = mouse;
//     // Transform mouse coordinates to grid space
//     if mx < SHAPE_LINE_COORD_X || my < SHAPE_LINE_COORD_Y {
//         return None;
//     }
//
//     let relative_x = mx - SHAPE_LINE_COORD_X;
//     let relative_y = my - SHAPE_LINE_COORD_Y;
//     // converting to the grid space
//     let (col, row) = (relative_x / CELL_SIZE, relative_y / CELL_SIZE);
//     let shapes_grid = shapes_as_grid(shapes);
//     let ix = rc_to_ix(row, col);
//
//     return shapes_grid.get(&ix)
//         .cloned()
//         .filter(|i| shapes.get(*i).is_some_and(|s| s.state == VISIBLE));
// }
//
// fn shapes_as_grid(shapes: &Vec<Shape>) -> HashMap<usize, usize> {
//     let mut result = HashMap::new();
//     for (i, s) in shapes.iter().enumerate() {
//         for (dx, dy) in Shape::cells(&s.kind) {
//             let n = rc_to_ix(dy, dx + s.x_cell_coordinate);
//             result.insert(n, i);
//         }
//     }
//
//     return result;
// }
//
// // converts row/col coordinate to the cell index
// fn rc_to_ix(r: usize, c: usize) -> usize {
//     return WIDTH_CELLS * r + c;
// }