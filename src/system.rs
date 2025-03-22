use std::collections::VecDeque;
use std::time::Duration;
use crate::events::{Event};
use crate::events::Event::{SelectedShapePlaced, ShapeSelected};
use crate::input;
use crate::game_entities::{GameState, ShapeType};
use crate::input::Input;
use crate::render::render::UserRenderConfig;
use crate::space_converters::{CellCoord, OffsetXY, to_cell_space, XY};

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
    );
}

pub struct SelectionValidationSystem;
impl System for SelectionValidationSystem {
    fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut VecDeque<Event>, render_config: &UserRenderConfig) {
        if input.mouse_right_clicked {
            state.deselect();
        }
        if let Some(XY(x, y)) = input.mouse_left_clicked {
            match &state.selected_shape {
                None => {
                    // nothing is selected, so we select shape from panel
                    let px = (x as f32) - render_config.panel_offset_x_px;
                    let py = (render_config.window_size.height as f32) - render_config.panel_offset_y_px - (y as f32);
                    if px < 0.0 || px > render_config.cell_size_px * (render_config.panel_cols as f32)
                        || py < 0.0 || py > render_config.cell_size_px * (render_config.panel_rows as f32) {
                        let col = (px / render_config.cell_size_px) as i16;
                        let row = (py / render_config.cell_size_px) as i16;
                        let over_shape = state.panel.shapes_in_cell_space.get(&CellCoord::new(col, row));
                        if let Some(&shape) = over_shape {
                            // shape coordinate in cell space
                            let x = &state.shape_choice;
                            //todo it's not cell coordinate, it's cell offset in cell space.
                            let shape_pos_0 = x.get(shape).expect("Invalid shape index").x_cell_coordinate;
                            let offset_x: i16 = (px - shape_pos_0).floor() as i16;
                            let offset_y: i16 = -py as i16;
                            events.push_front(ShapeSelected(shape, OffsetXY(offset_x, offset_y)))
                        }
                    }
                }
                // something was selected, and we try to place shape on the board
                Some(selected_shape) => {
                    let placement_xy_0 = XY(x, y).apply_offset(&selected_shape.anchor_offset);
                    let placement_0_cell = to_cell_space(XY(render_config.board_offset_x_px, render_config.board_offset_y_px),
                                  render_config.cell_size_px,
                                  render_config.window_size.height,
                                  placement_xy_0);

                    // we can always compute if placement is value to show the shadow
                    if state.is_valid_placement(&selected_shape.shape_type, &placement_0_cell) {
                        events.push_front(SelectedShapePlaced(selected_shape.shape_type, placement_0_cell))
                    }
                }
            }
        }
    }
}

// pub struct ValidationSystem;
// impl System for ValidationSystem {
//     fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut Vec<Event>) {
//         todo!()
//     }
// }
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