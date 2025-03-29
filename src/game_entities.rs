use crate::game_entities::ShapeState::VISIBLE;
use crate::space_converters::{CellCoord, OffsetXY};
use cgmath::num_traits::ToPrimitive;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::{thread_rng, Rng};
use std::cmp::{max, min};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Filled,
}

pub struct Board {
    pub grid: Vec<Cell>,
    pub size: usize,
}

impl Board {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            grid: vec![Cell::Empty; size * size],
            size,
        }
    }

    pub fn get(&self, col: usize, row: usize) -> Option<&Cell> {
        self.grid.get(row * self.size + col)
    }

    // Helper to fill a specific cell (for demo purposes)
    pub fn set_cell(&mut self, col: usize, row: usize, cell: Cell) {
        if let Some(slot) = self.grid.get_mut(row * self.size + col) {
            *slot = cell;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ShapeType {
    base_shape_type: BaseShapeType,
    mirror: bool,
    rotation: ShapeRot,
}
impl ShapeType {
    pub fn horizontal_cell_size(&self) -> i16 {
        let n = self.base_shape_type.dimensions();
        return match self.rotation {
            ShapeRot::No => n.horizontal,
            ShapeRot::Cw90 => n.vertical,
            ShapeRot::Cw180 => n.horizontal,
            ShapeRot::Cw270 => n.vertical,
        };
    }

    pub fn cells(&self) -> Vec<(usize, usize)> {
        let base_cells = self.base_shape_type.cells();
        let dimensions = self.base_shape_type.dimensions();
        let (w, h) = (dimensions.horizontal as usize, dimensions.vertical as usize);

        let transformed_cells: Vec<(usize, usize)> = base_cells
            .into_iter()
            .map(|(mut x, mut y)| {
                if self.mirror && w > 1 {
                    x = w - 1 - x;
                }

                let (x, y) = match self.rotation {
                    ShapeRot::No => (x, y),
                    ShapeRot::Cw90 => (y, w - 1 - x),
                    ShapeRot::Cw180 => (w - 1 - x, h - 1 - y),
                    ShapeRot::Cw270 => (h - 1 - y, x),
                };

                (x, y)
            })
            .collect();

        transformed_cells
    }
}

#[derive(Clone, Copy, PartialEq, Debug, EnumCount, EnumIter)]
pub enum ShapeRot {
    No,
    Cw90,
    Cw180,
    Cw270,
}

#[derive(Clone, Copy, PartialEq, Debug, EnumCount, EnumIter)]
pub enum BaseShapeType {
    T1,
    L1,
    I1,
    O,
    OO,
}

struct Dimension {
    horizontal: i16,
    vertical: i16,
}
impl Dimension {
    pub fn new(horizontal: i16, vertical: i16) -> Self {
        Dimension {
            horizontal,
            vertical,
        }
    }
}

impl BaseShapeType {
    pub fn dimensions(&self) -> Dimension {
        match self {
            BaseShapeType::T1 => Dimension::new(3, 2),
            BaseShapeType::L1 => Dimension::new(2, 3),
            BaseShapeType::I1 => Dimension::new(1, 4),
            BaseShapeType::O => Dimension::new(1, 1),
            BaseShapeType::OO => Dimension::new(2, 2),
        }
    }

    pub fn cells(&self) -> Vec<(usize, usize)> {
        return match self {
            // col , row
            BaseShapeType::T1 => vec![(1, 0), (0, 1), (1, 1), (2, 1)],

            BaseShapeType::L1 => vec![(0, 0), (0, 1), (0, 2), (1, 2)],

            BaseShapeType::I1 => vec![(0, 0), (0, 1), (0, 2), (0, 3)],

            BaseShapeType::O => vec![(0, 0)],
            BaseShapeType::OO => vec![(0, 0), (0, 1), (1, 0), (1, 1)],
        };
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Shape {
    pub kind: ShapeType,
    pub state: ShapeState,
    pub col_offset_in_panel_basis: i16, //todo extract relative position is useful for rendering
}

#[derive(Clone, PartialEq, Debug)]
pub enum ShapeState {
    VISIBLE,
    SELECTED,
    PLACED,
}

impl Shape {
    pub fn set_state(&mut self, state: ShapeState) {
        self.state = state;
    }

    pub fn new(kind: ShapeType, col_offset_in_panel_basis: i16) -> Shape {
        Shape {
            kind,
            state: VISIBLE,
            col_offset_in_panel_basis,
        }
    }

    pub fn get_random_choice(n: usize) -> Vec<Shape> {
        let mut rng = thread_rng(); // Random number generator
        let shapes: Vec<BaseShapeType> = BaseShapeType::iter().collect();

        let random_shapes: Vec<ShapeType> = (0..n)
            .map(|_| {
                let base_shape = shapes.choose(&mut rng).unwrap();
                let mirror = rng.gen_bool(0.5);
                let rotation = ShapeRot::iter().choose(&mut rng).unwrap();

                ShapeType {
                    base_shape_type: *base_shape,
                    mirror,
                    rotation,
                }
            })
            .collect();

        // Compute positions using a fold
        let mut current_col_offset = 0;
        random_shapes
            .into_iter()
            .map(|shape| {
                let position = current_col_offset;
                current_col_offset += shape.horizontal_cell_size() + 1; // Update for the next shape
                println!(
                    "generating start cell x {:?} for shape type  {:?}",
                    position, shape
                );
                return Shape::new(shape, position);
            })
            .collect()
    }
}

// todo Mb split into game and UI and system state (or even input). UI is a function of a game, but game - is what the logic is derived from
// and ui - what is actually rendered?
// system state - is whatever we need from the user. Like mouse position/last click position etc.Mb RNG comes here.
pub struct Game {
    pub board: Board,
    pub selected_shape: Option<SelectedShape>,
    pub stats: GameStats,

    pub panel: Panel,
    pub game_state: GameState,
}

pub struct SelectedShape {
    pub shape_type: ShapeType,
    //distance from selection point to top-left of the shape. So it must be always negative
    pub anchor_offset: OffsetXY,
}

pub struct Panel {
    pub shape_choice: Vec<Shape>,
    pub shapes_in_cell_space: HashMap<CellCoord, usize>,
}

impl Panel {
    fn from_shapes(shape_choice: Vec<Shape>) -> Self {
        let mut result: HashMap<CellCoord, usize> = HashMap::new();
        let mut offset_col = 0;
        let mut max_dx = 0;
        for (i, s) in shape_choice.iter().enumerate() {
            for (dx, dy) in s.kind.cells() {
                result.insert(CellCoord::new((dx + offset_col) as i16, dy as i16), i);
                max_dx = max(max_dx, dx)
            }
            offset_col = offset_col + 2 + max_dx;
            max_dx = 0;
        }

        return Panel {
            shape_choice,
            shapes_in_cell_space: result,
        };
    }

    pub fn generate_for_3() -> Self {
        let shapes = Shape::get_random_choice(3);
        Self::from_shapes(shapes)
    }
}

impl Game {
    pub fn new_level(board_size: usize, level: u16, total_score: i32) -> Self {
        // could go to level description
        let cells_filled = min(level as usize * 3 + 3, board_size * 3);
        let target_score = level as i32 * 10;

        let mut rng = thread_rng();

        let panel = Panel::generate_for_3();
        let mut board = Board::new(board_size);
        // Generate unique random cell coordinates
        let generated: Vec<(usize, usize)> = (0..board_size)
            .flat_map(|row| (0..board_size).map(move |col| (col, row)))
            .choose_multiple(&mut rng, cells_filled);

        // Fill the selected cells
        for (col, row) in generated {
            board.set_cell(col, row, Cell::Filled);
        }

        let stats = GameStats {
            level,
            target_score,
            current_score: 0,
            total_score,
        };

        Self {
            board,
            selected_shape: None,
            stats,
            panel,
            game_state: GameState::Playing,
        }
    }

    pub fn go_next_level(&mut self) {
        *self = Self::new_level(
            self.board.size,
            self.stats.level + 1,
            self.stats.total_score,
        );
    }

    pub fn is_valid_placement(&self, shape: &ShapeType, cell_coord: &CellCoord) -> bool {
        if cell_coord.col < 0 || cell_coord.row < 0 {
            return false;
        }
        let col = cell_coord.col.to_usize().unwrap();
        let row = cell_coord.row.to_usize().unwrap();
        for (dx, dy) in shape.cells() {
            let nx = col.wrapping_add(dx);
            let ny = row.wrapping_add(dy);
            if nx >= self.board.size || ny >= self.board.size {
                return false;
            }

            if self.board.get(nx, ny).is_none_or(|x| x == &Cell::Filled) {
                return false;
            }
        }
        true
    }

    pub fn place_shape(&mut self, shape_type: &ShapeType, cell_coord: &CellCoord) {
        assert!(
            cell_coord.row >= 0
                && cell_coord.row < self.board.size.to_i16().unwrap()
                && cell_coord.col >= 0
                && cell_coord.col < self.board.size.to_i16().unwrap(),
            "error placing cell out of the board {:?}",
            cell_coord
        );
        for (dx, dy) in shape_type.cells() {
            let col = cell_coord.col as usize + dx;
            let row = cell_coord.row as usize + dy;

            self.board.set_cell(col, row, Cell::Filled);
        }

        self.selected_shape = None;
        for s in self.panel.shape_choice.iter_mut() {
            if s.state == ShapeState::SELECTED {
                s.set_state(ShapeState::PLACED)
            }
        }
    }

    pub fn deselect(&mut self) {
        self.selected_shape = None;

        for s in self.panel.shape_choice.iter_mut() {
            if s.state == ShapeState::SELECTED {
                s.set_state(VISIBLE)
            }
        }
    }

    pub fn clean_row(&mut self, row: usize) {
        for col in 0..self.board.size {
            self.board.set_cell(col, row, Cell::Empty)
        }
    }

    pub fn clean_col(&mut self, col: usize) {
        for row in 0..self.board.size {
            self.board.set_cell(col, row, Cell::Empty)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GameState {
    Playing,
    GameOver,
    MoveToNextLevel,
}

pub struct GameStats {
    pub level: u16,
    pub target_score: i32,
    pub current_score: i32,
    pub total_score: i32,
}

#[cfg(test)]
mod tests {
    use crate::game_entities::BaseShapeType;

    use super::*;

    #[test]
    fn test_shapes_as_grid() {
        let shapes = vec![
            Shape::new(BaseShapeType::I2, 0),
            Shape::new(BaseShapeType::OO, 0),
        ];

        let result = Panel::from_shapes(shapes);

        let expected: HashMap<CellCoord, usize> = HashMap::from_iter(vec![
            // First shape (I)
            (CellCoord::new(0, 0), 0),
            (CellCoord::new(1, 0), 0),
            (CellCoord::new(2, 0), 0),
            (CellCoord::new(3, 0), 0),
            // Second shape (O) should be placed with an offset
            (CellCoord::new(5, 0), 0),
            (CellCoord::new(5, 1), 0),
            (CellCoord::new(6, 0), 0),
            (CellCoord::new(6, 1), 0),
        ]);

        assert_eq!(result.shapes_in_cell_space, expected);
    }
}
