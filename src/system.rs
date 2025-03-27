use std::collections::VecDeque;
use std::time::Duration;

use crate::events::Event;
use crate::events::Event::{SelectedShapePlaced, ShapeSelected};
use crate::game_entities::{Cell, GameState, Panel, ShapeState};
use crate::input;
use crate::input::Input;
use crate::render::render::UserRenderConfig;
use crate::space_converters::{CellCoord, OffsetXY, to_cell_space, within_bounds, XY};

// to think about:
// I have 3 spaces, which I need to convert from and to:
// 1. pixel screen space - represents coordinate of pixel on the screen
// 2. cell space - represents the cell number on panel or board
// 3. game space represents the game entities, like shape N 3.
// 4. also there's vertex/index space which I use for rendering :D
pub trait System {
    #[allow(unused_variables)]
    fn start(&mut self, state: &mut GameState) {}
    fn update_state(
        &self,
        input: &input::Input,
        dt: instant::Duration,
        state: &mut GameState,
        events: &mut VecDeque<Event>, // events so systems can communicate with each other
        render_config: &UserRenderConfig,
        event: Option<&Event>,
    );
}

pub struct SelectionValidationSystem;
impl System for SelectionValidationSystem {
    fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut VecDeque<Event>, render_config: &UserRenderConfig, oe: Option<&Event>) {
        if input.mouse_right_clicked {
            state.deselect();
        }
        if let Some(XY(x, y)) = input.mouse_left_clicked {
            match &state.selected_shape {
                None => {
                    // nothing is selected, so we select shape from panel
                    // coordinates of the mouse in the panel basis. Top-left is (0, 0).
                    let px = x - render_config.panel_offset_x_px;
                    let py = y - render_config.panel_offset_y_px;
                    println!("Clicking over normalized to panel offset {:?}, {:?} on panel", px, py);

                    if within_bounds(px, py,
                                     render_config.cell_size_px * (render_config.panel_cols as f32),
                                     render_config.cell_size_px * (render_config.panel_rows as f32)) {
                        let col = (px / render_config.cell_size_px) as i16;
                        let row = (py / render_config.cell_size_px) as i16;
                        println!("Clicking over {:?}, {:?} on panel", col, row);
                        // println!("Shapes on the panel space {:?}" , state.panel.shapes_in_cell_space.iter().);
                        let over_shape = state.panel.shapes_in_cell_space.get(&CellCoord::new(col, row));
                        if let Some(&shape_ix) = over_shape {
                            // shape coordinate in cell space
                            let available_shapes = &state.panel.shape_choice;
                            let shape = available_shapes.get(shape_ix).expect("Invalid shape index");

                            //todo it's not cell coordinate, it's cell offset in cell space.
                            if shape.state == ShapeState::VISIBLE {
                                // x coordinate in the panel basis
                                let shape_pos_0 = (shape.col_offset_in_panel_basis as f32) * render_config.cell_size_px;
                                let offset_x: i16 = (shape_pos_0 - px).floor() as i16;
                                let offset_y: i16 = -py as i16;
                                println!("Anchor offset ({:?}, {:?}). Shape zero x: {:?}", offset_x, offset_y, shape_pos_0);

                                events.push_front(ShapeSelected(shape_ix, OffsetXY(offset_x, offset_y)))
                            }
                        }
                    }
                }
                // something was selected, and we try to place shape on the board
                Some(selected_shape) => {
                    //todo CHECK THERE!
                    let placement_xy_0 = XY(x, y).apply_offset(&selected_shape.anchor_offset);
                    let placement_0_cell = to_cell_space(XY(render_config.board_offset_x_px, render_config.board_offset_y_px),
                                                         render_config.cell_size_px,
                                                         &placement_xy_0);

                    println!("Trying to place in the cell {:?}", &placement_0_cell);

                    // we can always compute if placement is value to show the shadow
                    if state.is_valid_placement(&selected_shape.shape_type, &placement_0_cell) {
                        events.push_front(SelectedShapePlaced(selected_shape.shape_type, placement_0_cell))
                    }
                }
            }
        }
    }
}

pub struct PlacementSystem;
impl System for PlacementSystem {
    fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut VecDeque<Event>, render_config: &UserRenderConfig, event: Option<&Event>) {
        if let Some(SelectedShapePlaced(shape, cell)) = event {
            println!("Placing shape {:?} to {:?}", shape, cell);
            // update board
            state.place_shape(shape, cell);

            if (state.panel.shape_choice.iter().all(|s| s.state != ShapeState::VISIBLE)) {
                state.panel = Panel::generate_for_3();
            }
        }
    }
}

// checks the board state after end of turn, that
// 1. if there's some row or column that is filled (or some other  shape)
// 2. cleans the board
// 3. increment score
pub struct ScoreCleanupSystem;
impl System for ScoreCleanupSystem {
    fn update_state(&self, input: &Input, dt: Duration, game: &mut GameState, events: &mut VecDeque<Event>, render_config: &UserRenderConfig, event: Option<&Event>) {
        let size = game.board.size;

        let mut row_counts = vec![0; size];
        let mut col_counts = vec![0; size];

        let mut total_cells = 0;
        let mut full_cols = 0;
        let mut full_rows = 0;

        for row in 0..size {
            for col in 0..size {
                if game.board.get(col, row).is_some_and(|x| x == &Cell::Filled) {
                    row_counts[row] += 1;
                    col_counts[col] += 1;
                }
            }
        }

        for row in 0..size {
            if row_counts[row] == size {
                full_rows += 1;
                total_cells += size;

                game.clean_row(row);
            }
        }
        for col in 0..size {
            if col_counts[col] == size {
                full_cols += 1;
                total_cells += size;

                game.clean_col(col);
            }
        }

        //todo we can extract the score math in the different system, so we could extend the way score is computed
        game.score = game.score + (total_cells + full_cols * size + full_rows * size) as u32
    }
}
//
//
// pub struct ShapeGenerationSystem;
// impl System for ShapeGenerationSystem {
//     fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut Vec<Event>) {
//         todo!()
//     }
// }
//
// // places shapes on the board
// // removes from the board
// pub struct BoardSystem;
// impl System for BoardSystem {
//     fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut Vec<Event>) {
//         todo!()
//     }
// }
//
//
// pub struct ScoreSystem;
// impl System for ScoreSystem {
//     fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut Vec<Event>) {
//
//     }
// }