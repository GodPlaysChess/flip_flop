use std::cmp::max;
use glyphon::Cursor;
use crate::events::XY;
use crate::game_entities::{Board, Cell, Shape};
use crate::render::buffer::{PANEL_HEIGTH, PANEL_WIDTH};

// the UI contains only visible elements. I.e only things are to be rendered.
// i.e. if shape is hidden - it's not in the UI. Treat it like intermediate datastructure
// between game state and vertex information that is passed in shader
struct UI {
    board: Board,
    panel: Panel,
    mouse: MousePosition,
    score: Score,
}

struct Score {
    value: u16,
}

struct Panel {
    shapes: Vec<Shape>,
}

struct MousePosition {
    xy: XY,
}

//shapes -> index_buffer
pub fn render_panel(shapes: &Vec<Shape>) -> Vec<u32> {

    // convert shapes to cells
    let grid = shapes_to_cell_space(shapes);
    // convert grid + dimensions to indices for triangles
    return to_index_space(grid, PANEL_WIDTH, PANEL_HEIGTH);
    // same thing in the board -> extract to the common thing
}

pub fn to_index_space(cells: Vec<(usize, usize)>, max_col: usize, max_row: usize) -> Vec<u32> {
    let mut indices = Vec::new();

    for (x, y) in cells {
        let top_left = (y * (max_col + 1) + x ) as u32;
        let top_right = top_left + 1;
        let bottom_left = top_left + (max_col as u32 + 1);
        let bottom_right = bottom_left + 1;

        // Two triangles per cell (diagonal split)
        indices.extend_from_slice(&[
            top_left, bottom_left, bottom_right, // First triangle
            top_left, bottom_right, top_right,  // Second triangle
        ]);
    }
    println!("panel: {:?}", indices);
    indices
}


// in cell space, converts the list of shapes to list of coords in a format of
// col -> row from top to bottom
pub fn shapes_to_cell_space(shapes: &Vec<Shape>) -> Vec<(usize, usize)> {
    let mut result: Vec<(usize, usize)> = Vec::new();
    let mut offset_col = 0;
    let mut max_dx = 0;
    for (i, s) in shapes.iter().enumerate() {
        for (dx, dy) in Shape::cells(&s.kind) {
            result.push((dx + offset_col, dy));
            max_dx = max(max_dx, dx)
        }
        offset_col = offset_col + 2 + max_dx;
        max_dx = 0;
    }

    return result;
}

// board to index buffer; todo could be the same method -> just render cells
pub fn render_board(board: &Board) -> Vec<u32> {
    let mut indices = Vec::new();
    let board_size = board.grid.len();

    /*
             0   1   2   3
               C0  C1  C2
             4   5   6   7
               C3  C4  C5
             8   9   10  11
               C6  C7  C8
             12  13  14  15

     */
    for row in 0..board_size {
        for col in 0..board_size {
            if let Cell::Filled = board.grid[row][col] {
                let top_left = (row * (board_size + 1) + col) as u32;
                let top_right = top_left + 1;
                let bottom_left = top_left + (board_size + 1) as u32;
                let bottom_right = bottom_left + 1;

                // Two triangles per cell (diagonal split)
                indices.extend_from_slice(&[
                    top_left, bottom_left, bottom_right, // First triangle
                    top_left, bottom_right, top_right,  // Second triangle
                ]);
            }
        }
    }

    indices
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

    #[test]
    fn test_single_cell() {
        let cells = vec![(0, 0)]; // Top-left corner
        let indices = to_index_space(cells, 7, 3);

        assert_eq!(indices, vec![
            0, 8, 9,  // First triangle
            0, 9, 1,  // Second triangle
        ]);
    }

    #[test]
    fn test_two_adjacent_cells_horizontally() {
        let cells = vec![(0, 0), (1, 0)]; // Two side-by-side cells in row 0
        let indices = to_index_space(cells, 7, 3);

        assert_eq!(indices, vec![
            0, 8, 9,  0, 9, 1,  // First cell
            1, 9, 10, 1, 10, 2, // Second cell
        ]);
    }

    #[test]
    fn test_two_adjacent_cells_vertically() {
        let cells = vec![(0, 0), (0, 1)]; // Two stacked cells
        let indices = to_index_space(cells, 7, 3);

        assert_eq!(indices, vec![
            0, 8, 9,  0, 9, 1,  // First cell
            8, 16, 17, 8, 17, 9, // Second cell (below first one)
        ]);
    }

    #[test]
    fn test_non_contiguous_cells_in_elonagated_grid() {
        let cells = vec![(0, 0), (2, 1), (5, 2)]; // Scattered cells
        let indices = to_index_space(cells, 7, 3);

        assert_eq!(indices, vec![
            0, 8, 9,   0, 9, 1,   // First cell (0,0)
            10, 18, 19, 10, 19, 11, // Second cell (2,1)
            21, 29, 30, 21, 30, 22, // Third cell (5,2)
        ]);
    }
}