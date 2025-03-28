use crate::game_entities::ShapeType;
use crate::space_converters::{CellCoord, OffsetXY};

#[derive(Debug, Clone)]
pub enum Event {
    ShapeSelected(usize, OffsetXY),
    SelectedShapePlaced(ShapeType, CellCoord),
}
