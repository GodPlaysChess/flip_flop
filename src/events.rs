use crate::game_entities::{Cell, ShapeType};
use crate::space_converters::{CellCoord, OffsetXY};

#[derive(Debug, Clone)]
pub enum Event {
    //-- background events
    ScoreUpdated(u32),
    BoardUpdated(Vec<BoardUpdate>),
    // shape select/deselect (disappearance from the shapes list)
    ShapeSelected(usize, OffsetXY),

    // based on this event we update board, update score and play sound, and may be even remove the whole row
    SelectedShapePlaced(ShapeType, CellCoord),
    // -- foreground events
    // mouse moved with shape selected
    // board highlight

    // system events
    FocusChanged,
    ButtonPressed,
    Resize(f32, f32),
    Nothing
}



#[derive(Debug, Copy, Clone)]
pub struct BoardUpdate {
    pub cell: Cell,
    pub coord: CellCoord,
}

