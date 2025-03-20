use std::time::Duration;
use crate::events::{Event, XY};
use crate::events::Event::ShapeSelected;
use crate::input;
use crate::game_entities::GameState;
use crate::input::Input;
use crate::render::render::UserRenderConfig;

pub trait System {
    #[allow(unused_variables)]
    fn start(&mut self, state: &mut GameState) {}
    fn update_state(
        &self,
        input: &input::Input,
        dt: instant::Duration,
        state: &mut GameState,
        events: &mut Vec<Event>, // events so systems can communicate with each other
        render_config: &UserRenderConfig,
    );
}

pub struct PlacementSystem;
impl System for PlacementSystem {
    fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut Vec<Event>, render_config: &UserRenderConfig) {
        if input.mouse_right_clicked {
            state.deselect();
        }
        // nothing is selected, so we select shape from panel
        if state.selected_shape.is_none() {
            if let Some(XY(x, y)) = input.mouse_left_clicked {
                let px = (x as f32) - render_config.panel_offset_x_px;
                let py = (render_config.window_size.height as f32) - render_config.panel_offset_y_px - (y as f32);
                if px < 0.0 || px > render_config.cell_size_px * (render_config.panel_cols as f32)
                    || py < 0.0 || py > render_config.cell_size_px * (render_config.panel_rows as f32) {
                    let col = (px / render_config.cell_size_px) as usize;
                    let row = (py / render_config.cell_size_px) as usize;
                    let over_shape = state.panel.shapes_in_cell_space.get(&(col, row));
                    if let Some(shape) = over_shape {
                        events.push(ShapeSelected(shape.clone()))
                    }
                }
            }
        }

        // something was selected, and we try to place shape on the board
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