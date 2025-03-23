use std::cmp::max;
use std::collections::{HashMap};
use cgmath::num_traits::ToPrimitive;
use rand::prelude::SliceRandom;
use crate::game_entities::ShapeState::VISIBLE;
use crate::space_converters::{CellCoord, OffsetXY};
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

#[derive(Clone, Copy, PartialEq, Debug, EnumCount, EnumIter)]
pub enum ShapeType {
    T1,
    T2,
    T3,
    T4,
    L1,
    L2,
    L3,
    L4,
    I1,
    I2,
    O,
    OO,
}

impl ShapeType {
    pub fn horizontal_cell_size(&self) -> i16 {
        match self {
            ShapeType::T1 => 3,
            ShapeType::T2 => 2,
            ShapeType::T3 => 3,
            ShapeType::T4 => 2,
            ShapeType::L1 => 2,
            ShapeType::L2 => 3,
            ShapeType::L3 => 2,
            ShapeType::L4 => 3,
            ShapeType::I1 => 1,
            ShapeType::I2 => 4,
            ShapeType::O => 1,
            ShapeType::OO => 2,
        }
    }

    // todo apply random symmetry: mirror, rotational
    pub fn cells(&self) -> Vec<(usize, usize)> {
        return match self {
            // col , row
            ShapeType::T1 => vec![(1, 0), (0, 1), (1, 1), (2, 1)],
            ShapeType::T2 => vec![(0, 0), (0, 1), (0, 2), (1, 1)],
            ShapeType::T3 => vec![(0, 0), (1, 0), (1, 1), (2, 0)],
            ShapeType::T4 => vec![(0, 1), (1, 0), (1, 1), (1, 2)],

            ShapeType::L1 => vec![(0, 0), (0, 1), (0, 2), (1, 2)],
            ShapeType::L2 => vec![(0, 1), (1, 1), (2, 0), (2, 1)],
            ShapeType::L3 => vec![(0, 0), (1, 0), (1, 1), (1, 2)],
            ShapeType::L4 => vec![(0, 0), (0, 1), (1, 0), (2, 0)],

            ShapeType::I1 => vec![(0, 0), (0, 1), (0, 2), (0, 3)],
            ShapeType::I2 => vec![(0, 0), (1, 0), (2, 0), (3, 0)],

            ShapeType::O => vec![(0, 0)],
            ShapeType::OO => vec![(0, 0), (0, 1), (1, 0), (1, 1)],
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
        let mut rng = rand::thread_rng(); // Random number generator
        let shapes: Vec<ShapeType> = ShapeType::iter().collect();

        let random_shapes: Vec<&ShapeType> = (0..n)
            //todo change to derived value
            .map(|_| shapes.choose(&mut rng).unwrap())
            .collect();

        // Compute positions using a fold
        let mut current_col_offset = 0;
        random_shapes
            .into_iter()
            .map(|shape| {
                let position = current_col_offset;
                current_col_offset += shape.horizontal_cell_size() + 1; // Update for the next shape
                println!("generating start cell x {:?} for shape type  {:?}", position, shape);
                return Shape::new(*shape, position);
            })
            .collect()
    }
}

// todo Mb split into game and UI and system state (or even input). UI is a function of a game, but game - is what the logic is derived from
// and ui - what is actually rendered?
// system state - is whatever we need from the user. Like mouse position/last click position etc.Mb RNG comes here.
pub struct GameState {
    pub board: Board,
    pub selected_shape: Option<SelectedShape>,
    pub score: u32,

    // this one is not really game state. It's like UI or smth. also it's XY
    pub mouse_position: (usize, usize),
    pub last_click_position: (usize, usize),
    pub panel: Panel,
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

        return Panel { shape_choice, shapes_in_cell_space: result };
    }
}

impl GameState {
    pub fn new(board_size: usize) -> Self {
        let shapes = Shape::get_random_choice(3);
        let panel = Panel::from_shapes(shapes);
        Self {
            board: Board::new(board_size),
            selected_shape: None,
            score: 0,
            mouse_position: (0, 0),
            last_click_position: (0, 0),
            panel,
        }
    }

    pub fn is_valid_placement(&self, shape: &ShapeType, cell_coord: &CellCoord) -> bool {
        if cell_coord.row < 0 || cell_coord.row >= self.board.size.to_i16().unwrap() &&
            cell_coord.col < 0 || cell_coord.col >= self.board.size.to_i16().unwrap() {
            return false;
        }
        let col = cell_coord.col.to_usize().unwrap();
        let row = cell_coord.row.to_usize().unwrap();

        for (dx, dy) in shape.cells() {
            let nx = col.wrapping_add(dx);
            let ny = row.wrapping_add(dy);
            if self.board.get(nx, ny).is_none_or(|x| x == &Cell::Filled) {
                return false;
            }
        }
        true
    }

    pub fn place_shape(&mut self, shape_type: &ShapeType, cell_coord: &CellCoord) {
        assert!(cell_coord.row >= 0 && cell_coord.row < self.board.size.to_i16().unwrap() &&
                    cell_coord.col >= 0 && cell_coord.col < self.board.size.to_i16().unwrap(),
                "error placing cell out of the board {:?}", cell_coord);
        for (dx, dy) in shape_type.cells() {
            let col = cell_coord.col as usize + dx;
            let row = cell_coord.row as usize + dy;

            &mut self.board.set_cell(col, row, Cell::Filled);
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

#[cfg(test)]
mod tests {
    use crate::game_entities::ShapeType;

    use super::*;

    #[test]
    fn test_shapes_as_grid() {
        let shapes = vec![
            Shape::new(ShapeType::I2, 0.0),
            Shape::new(ShapeType::OO, 0.0),
        ];

        let result = Panel::from_shapes(shapes);

        let expected = vec![
            // First shape (I)
            (0, 0), (1, 0), (2, 0), (3, 0),
            // Second shape (O) should be placed with an offset
            (5, 0), (5, 1), (6, 0), (6, 1),
        ];

        assert_eq!(result.shapes_in_cell_space, expected);
    }
}