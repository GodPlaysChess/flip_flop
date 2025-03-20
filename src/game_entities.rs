use std::cmp::max;
use std::collections::{HashMap, HashSet, VecDeque};
use rand::Rng;
use crate::events::{BoardUpdate, CellCoord, Event};
use crate::events::Event::BoardUpdated;
use crate::game_entities::ShapeState::VISIBLE;

pub const BOARD_SIZE: usize = 10; //
pub const CELL_SIZE: usize = 40; // Size of each cell in pixels

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


#[derive(Clone, PartialEq, Debug)]
pub struct Shape {
    pub kind: ShapeType,
    pub state: ShapeState,
    pub x_cell_coordinate: usize, // relative position is useful for rendering
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

    pub fn new(kind: ShapeType, x: usize) -> Shape {
        Shape {
            kind,
            state: VISIBLE,
            x_cell_coordinate: x,
        }
    }

    pub fn get_random_choice(n: usize) -> Vec<Shape> {
        let mut rng = rand::thread_rng(); // Random number generator
        let all_shapes = [
            (ShapeType::T1, 3),
            (ShapeType::T2, 2),
            (ShapeType::T3, 3),
            (ShapeType::T4, 2),
            (ShapeType::L1, 2),
            (ShapeType::L2, 3),
            (ShapeType::L3, 2),
            (ShapeType::L4, 3),
            (ShapeType::I1, 1),
            (ShapeType::I2, 4),
            (ShapeType::O, 1),
            (ShapeType::OO, 2),
        ];

        let random_shapes: Vec<(ShapeType, usize)> = (0..n)
            .map(|_| all_shapes[rng.gen_range(0..all_shapes.len())])
            .collect();

        // Compute positions using a fold
        let mut current_position = 0;
        random_shapes
            .into_iter()
            .map(|(shape, size)| {
                let position = current_position;
                current_position += size + 1; // Update for the next shape
                return Shape::new(shape, position);
            })
            .collect()
    }

    // todo apply random symmetry: mirror, rotational
    pub fn cells(kind: &ShapeType) -> Vec<(usize, usize)> {
        return match kind {
            // col , row
            ShapeType::T1 => vec![(1, 0), (0, 1), (1, 1), (2, 1)],
            ShapeType::T2 => vec![(0, 0), (0, 1), (0, 2), (1, 1)],
            ShapeType::T3 => vec![(0, 0), (1, 0), (1, 1), (2, 0)],
            ShapeType::T4 => vec![(0, 1), (1, 0), (1, 1), (1, 2)],

            ShapeType::L1 => vec![(0, 0), (0, 1), (0, 2), (1, 2)],
            ShapeType::L2 => vec![(0, 1), (1, 1), (2, 0), (2, 1)],
            ShapeType::L3 => vec![(0, 0), (1, 0), (1, 1), (1, 2)],
            ShapeType::L4 => vec![(0, 0), (0, 1), (1, 0), (2, 0)],

            ShapeType::I1 => vec![(0, 1), (0, 2), (0, 3), (0, 4)],
            ShapeType::I2 => vec![(0, 0), (1, 0), (2, 0), (3, 0)],

            ShapeType::O => vec![(0, 0)],
            ShapeType::OO => vec![(0, 0), (0, 1), (1, 0), (1, 1)],
        };
    }

    pub fn horizontal_size(kind: &ShapeType) -> usize {
        return match kind {
            ShapeType::T1 => 3,
            ShapeType::T2 => 2,
            ShapeType::T3 => 3,
            ShapeType::T4 => 2,
            ShapeType::L1 => 3,
            ShapeType::L2 => 2,
            ShapeType::L3 => 3,
            ShapeType::L4 => 2,
            ShapeType::I1 => 1,
            ShapeType::I2 => 4,
            ShapeType::O => 1,
            ShapeType::OO => 2,
        };
    }
}

// todo Mb split into game and UI and system state (or even input). UI is a function of a game, but game - is what the logic is derived from
// and ui - what is actually rendered?
// system state - is whatever we need from the user. Like mouse position/last click position etc.Mb RNG comes here.
pub struct GameState {
    pub board: Board,
    pub shape_choice: Vec<Shape>,
    pub selected_shape: Option<ShapeType>,
    pub score: u32,

    // this one is not really game state. It's like UI or smth.
    pub mouse_position: (usize, usize),
    pub last_click_position: (usize, usize),
    pub panel: Panel,
}

pub struct Panel {
    pub shapes_in_cell_space: HashMap<(usize, usize), usize>,
}

impl GameState {
    pub fn new(board_size: usize) -> Self {
        let shapes = Shape::get_random_choice(3);
        let panel = shapes_to_cell_space(&shapes);
        Self {
            board: Board::new(board_size),
            shape_choice: Shape::get_random_choice(3),
            selected_shape: None,
            score: 0,
            mouse_position: (0, 0),
            last_click_position: (0, 0),
            panel,
        }
    }


    pub fn is_valid_placement_of_selected_shape(&self) -> bool {
        if let Some(kind) = &self.selected_shape {
            let (x, y) = &self.mouse_position;
            let n = x / CELL_SIZE;
            let m = y / CELL_SIZE;
            return self.is_valid_placement(kind, n, m);
        }
        return false;
    }

    fn is_valid_placement(&self, shape: &ShapeType, col: usize, row: usize) -> bool {
        for (dx, dy) in Shape::cells(shape) {
            let nx = col.wrapping_add(dx);
            let ny = row.wrapping_add(dy);
            if nx >= BOARD_SIZE || ny >= BOARD_SIZE || self.board.get(nx, ny).is_none_or(|x| x == &Cell::Filled) {
                return false;
            }
        }
        true
    }

    pub fn place_shape(&mut self, events: &mut VecDeque<Event>) {
        if let Some(selected_shape) = self.selected_shape {
            let mut updates: Vec<BoardUpdate> = vec![];
            for (dx, dy) in Shape::cells(&selected_shape) {
                let (x, y) = &self.mouse_position;
                let n = x / CELL_SIZE;
                let m = y / CELL_SIZE;
                let nx = n.wrapping_add(dx);
                let ny = m.wrapping_add(dy);
                if nx < BOARD_SIZE && ny < BOARD_SIZE &&
                    self.board.get(nx, ny).is_some_and(|x| x == &Cell::Filled) {
                    updates.push(BoardUpdate {
                        cell: Cell::Filled,
                        coord: CellCoord(nx, ny),
                    }
                    )
                }
            }

            if !updates.is_empty() {
                events.push_front(BoardUpdated(updates))
            }

            self.deplace()
        }
    }

    fn deplace(&mut self) {
        self.selected_shape = None;
        for s in self.shape_choice.iter_mut() {
            if s.state == ShapeState::SELECTED {
                s.set_state(ShapeState::PLACED)
            }
        }
    }

    pub fn deselect(&mut self) {
        self.selected_shape = None;

        for s in self.shape_choice.iter_mut() {
            if s.state == ShapeState::SELECTED {
                s.set_state(VISIBLE)
            }
        }
    }
}

// in cell space, converts the list of shapes to list of coords in a format of
// col -> row from top to bottom
fn shapes_to_cell_space(shapes: &Vec<Shape>) -> Panel {
    let mut result: HashMap<(usize, usize), usize> = HashMap::new();
    let mut offset_col = 0;
    let mut max_dx = 0;
    for (i, s) in shapes.iter().enumerate() {
        for (dx, dy) in Shape::cells(&s.kind) {
            result.insert((dx + offset_col, dy), i);
            max_dx = max(max_dx, dx)
        }
        offset_col = offset_col + 2 + max_dx;
        max_dx = 0;
    }

    return Panel { shapes_in_cell_space: result };
}

#[cfg(test)]
mod tests {
    use crate::game_entities::ShapeType;

    use super::*;

    #[test]
    fn test_shapes_as_grid() {
        let shapes = vec![
            Shape::new(ShapeType::I2, 0),
            Shape::new(ShapeType::OO, 0),
        ];

        let result = shapes_to_cell_space(&shapes);

        let expected = vec![
            // First shape (I)
            (0, 0), (1, 0), (2, 0), (3, 0),
            // Second shape (O) should be placed with an offset
            (5, 0), (5, 1), (6, 0), (6, 1),
        ];

        assert_eq!(result, expected);
    }
}