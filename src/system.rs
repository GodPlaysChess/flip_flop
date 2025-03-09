use std::time::Duration;
use crate::events::Event;
use crate::input;
use crate::game_entities::GameState;
use crate::input::Input;

pub trait System {
    #[allow(unused_variables)]
    fn start(&mut self, state: &mut GameState) {}
    fn update_state(
        &self,
        input: &input::Input,
        dt: instant::Duration,
        state: &mut GameState,
        events: &mut Vec<Event>,
    );
}

pub struct ValidationSystem;

impl System for ValidationSystem {
    fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut Vec<Event>) {
        todo!()
    }
}


pub struct ShapeGenerationSystem;
impl System for ShapeGenerationSystem {
    fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut Vec<Event>) {
        todo!()
    }
}

// places shapes on the board
// removes from the board
pub struct BoardSystem;
impl System for BoardSystem {
    fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut Vec<Event>) {
        todo!()
    }
}


pub struct ScoreSystem;
impl System for ScoreSystem {
    fn update_state(&self, input: &Input, dt: Duration, state: &mut GameState, events: &mut Vec<Event>) {


    }
}