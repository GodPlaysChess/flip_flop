use winit::{event::ElementState, keyboard::KeyCode};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use crate::events::XY;

// todo this one just data structure to pass relevant input to the logic.
// in omy case the relevant parts are:
// mouse clicked, mouse coords
#[derive(Debug, Default)]
pub struct Input {
    pub esc_pressed: bool,
    pub mouse_left_clicked: Option<XY>,
    pub mouse_right_clicked: bool,
    pub mouse_position: XY,
}

impl Input {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update_kb(&mut self, key: &KeyCode, state: &ElementState) -> bool {
        let pressed = state.is_pressed();
        match key {
            KeyCode::Escape => {
                self.esc_pressed = pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_mouse(&mut self, button: &MouseButton, state: &ElementState) -> bool {
        let pressed = state.is_pressed();
        match button {
            MouseButton::Left => {
                println!("Left mouse button clicked at {:?}", self.mouse_position);
                self.mouse_left_clicked = Some(self.mouse_position.clone()).filter(|x| pressed);
                true
            }
            MouseButton::Right => {
                println!("Right mouse button clicked at {:?}", self.mouse_position.clone());
                self.mouse_right_clicked = true;
                true
            }
            _ => false
        }
    }

    pub fn update_mouse_position(&mut self, position: PhysicalPosition<f64>) {
        self.mouse_position = XY(position.x as usize, position.y as usize);
    }

    pub fn reset(&mut self) {
        self.mouse_left_clicked = None;
        self.mouse_right_clicked = false;
    }
}