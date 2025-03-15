use std::collections::VecDeque;
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
    pub grid: [[Cell; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub(crate) fn new() -> Self {
        Self {
            grid: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    // Helper to fill a specific cell (for demo purposes)
    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        if x < BOARD_SIZE && y < BOARD_SIZE {
            self.grid[y][x] = cell;
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

pub struct GameState {
    pub board: Board,
    pub shape_choice: Vec<Shape>,
    pub selected_shape: Option<ShapeType>,
    pub score: u32,
    pub mouse_position: (usize, usize),
    pub last_click_position: (usize, usize),
}

impl GameState {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            shape_choice: Shape::get_random_choice(3),
            selected_shape: None,
            score: 0,
            mouse_position: (0, 0),
            last_click_position: (0, 0),
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

    fn is_valid_placement(&self, shape: &ShapeType, n: usize, m: usize) -> bool {
        for (dx, dy) in Shape::cells(shape) {
            let nx = n.wrapping_add(dx);
            let ny = m.wrapping_add(dy);
            if nx >= BOARD_SIZE || ny >= BOARD_SIZE || self.board.grid[ny][nx] != Cell::Empty {
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
                if nx < BOARD_SIZE && ny < BOARD_SIZE {
                    self.board.grid[ny][nx] = Cell::Filled;
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