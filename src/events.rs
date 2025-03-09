use crate::game_entities::Cell;

#[derive(Debug, Clone)]
pub enum Event {
    //-- background events
    ScoreUpdated(u32),
    BoardUpdated(Vec<BoardUpdate>),
    // shape select/deselect (disappearance from the shapes list)
    ShapeChoiceUpdate,
    // -- foreground events
    // mouse moved with shape selected
    // board highlight

    // system events
    FocusChanged,
    ButtonPressed,
    Resize(f32, f32)
}

// pixel coordinates
#[derive(Debug, Default)]
pub struct XY(pub usize, pub usize);

// cell coordinate on the board, i.e. row, col pair.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct CellCoord(pub usize,pub usize);

#[derive(Debug, Copy, Clone)]
pub struct BoardUpdate {
    pub cell: Cell,
    pub coord: CellCoord,
}

