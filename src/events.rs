use crate::game_entities::{Cell, Shape, ShapeType};

#[derive(Debug)]
pub enum Event {
    //-- background events
    ScoreUpdated(u32),
    BoardUpdated(Vec<BoardUpdate>),
    // shape select/deselect (disappearance from the shapes list)
    ShapeChoiceUpdate
    // -- foreground events
    // mouse moved with shape selected
    // board highlight
}

// pixel coordinates
#[derive(Debug)]
pub struct XY(pub usize, pub usize);

// cell coordinate on the board, i.e. row, col pair.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct CellCoord(pub usize,pub usize);

#[derive(Debug)]
pub struct BoardUpdate {
    pub cell: Cell,
    pub coord: CellCoord,
}

