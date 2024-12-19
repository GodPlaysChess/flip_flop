pub const BOARD_SIZE: usize = 12; // 12x12 board
pub const CELL_SIZE: usize = 40; // Size of each cell in pixels

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
pub enum ShapeType {
    T,
    L,
}

pub struct Shape {
    pub kind: ShapeType,
    pub cells: Vec<(usize, usize)>, // Relative positions of filled cells
    pub bot_left_pos: (usize, usize),
}

impl Shape {
    pub fn new(kind: ShapeType, pos: (usize, usize)) -> Self {
        // let cells = match kind {
        //     ShapeType::T => vec![(1, 0), (0, 1), (1, 1), (2, 1)], // T-shape
        //     ShapeType::L => vec![(0, 0), (0, 1), (0, 2), (1, 2)], // L-shape
        // };
        let c = Self::cells(&kind);
        Self { kind, cells: c, bot_left_pos: pos }
    }

    pub fn cells(kind: &ShapeType) -> Vec<(usize, usize)> {
        return match kind {
            ShapeType::T => vec![(1, 0), (0, 1), (1, 1), (2, 1)], // T-shape
            ShapeType::L => vec![(0, 0), (0, 1), (0, 2), (1, 2)], // L-shape
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
        let shape_choice = vec![
            Shape::new(ShapeType::T, (100, 500)),
            Shape::new(ShapeType::L, (300, 500)),
        ];
        Self {
            board: Board::new(),
            shape_choice,
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

    pub fn place_shape(&mut self) {
        if let Some(selected_shape) = self.selected_shape {
            for (dx, dy) in Shape::cells(&selected_shape) {
                let (x, y) = &self.mouse_position;
                let n = x / CELL_SIZE;
                let m = y / CELL_SIZE;
                let nx = n.wrapping_add(dx);
                let ny = m.wrapping_add(dy);
                if nx < BOARD_SIZE && ny < BOARD_SIZE {
                    self.board.grid[ny][nx] = Cell::Filled;
                }
            }
        }
        self.score += 4;
    }
}