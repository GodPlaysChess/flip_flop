use std::collections::HashMap;
use crate::events::XY;
use crate::game_entities::{Board, Cell, Panel, Shape};

// the UI contains only visible elements. I.e only things are to be rendered.
// i.e. if shape is hidden - it's not in the UI. Treat it like intermediate datastructure
// between game state and vertex information that is passed in shader
struct UI {
    board: Board,
    mouse: MousePosition,
    score: Score,
}

struct Score {
    value: u16,
}

struct MousePosition {
    xy: XY,
}

//shapes -> index_buffer
pub fn render_panel(panel: &Panel, panel_width_cols: usize) -> Vec<u32> {
    // convert grid + dimensions to indices for triangles
    return to_index_space(&panel.shapes_in_cell_space, panel_width_cols);
    // same thing in the board -> extract to the common thing
}

pub fn to_index_space(cells: &HashMap<(usize, usize), usize>, max_col: usize) -> Vec<u32> {
    let mut indices = Vec::new();

    for (x, y) in cells.keys() {
        indices.extend(cell_to_ix(x, y, max_col));
    }
    indices
}

fn cell_to_ix(col: &usize, row: &usize, max_col: usize) -> [u32; 6] {
    let top_left = (row * (max_col + 1) + col) as u32;
    let top_right = top_left + 1;
    let bottom_left = top_left + (max_col as u32 + 1);
    let bottom_right = bottom_left + 1;
    return [
        top_left, bottom_left, bottom_right, // First triangle
        top_left, bottom_right, top_right,  // Second triangle
    ];
}



// board to index buffer
pub fn render_board(board: &Board) -> Vec<u32> {
    let mut indices = Vec::new();

    /*
             0   1   2   3
               C0  C1  C2
             4   5   6   7
               C3  C4  C5
             8   9   10  11
               C6  C7  C8
             12  13  14  15

     */
    for row in 0..board.size {
        for col in 0..board.size {
            if board.get(col, row).is_some_and(|x| x == &Cell::Filled) {
                indices.extend(cell_to_ix(&col, &row, board.size));
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
    fn test_single_cell() {
        let cells = vec![(0, 0)]; // Top-left corner
        let indices = to_index_space(&cells, 7);

        assert_eq!(indices, vec![
            0, 8, 9,  // First triangle
            0, 9, 1,  // Second triangle
        ]);
    }

    #[test]
    fn test_two_adjacent_cells_horizontally() {
        let cells = vec![(0, 0), (1, 0)]; // Two side-by-side cells in row 0
        let indices = to_index_space(&cells, 7);

        assert_eq!(indices, vec![
            0, 8, 9, 0, 9, 1,  // First cell
            1, 9, 10, 1, 10, 2, // Second cell
        ]);
    }

    #[test]
    fn test_two_adjacent_cells_vertically() {
        let cells = vec![(0, 0), (0, 1)]; // Two stacked cells
        let indices = to_index_space(&cells, 7);

        assert_eq!(indices, vec![
            0, 8, 9, 0, 9, 1,  // First cell
            8, 16, 17, 8, 17, 9, // Second cell (below first one)
        ]);
    }

    #[test]
    fn test_non_contiguous_cells_in_elonagated_grid() {
        let cells = vec![(0, 0), (2, 1), (5, 2)]; // Scattered cells
        let indices = to_index_space(&cells, 7);

        assert_eq!(indices, vec![
            0, 8, 9, 0, 9, 1,   // First cell (0,0)
            10, 18, 19, 10, 19, 11, // Second cell (2,1)
            21, 29, 30, 21, 30, 22, // Third cell (5,2)
        ]);
    }
}