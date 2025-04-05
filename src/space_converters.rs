use crate::game_entities::ShapeState::VISIBLE;
use crate::game_entities::{Board, Cell, Panel};
use crate::render::render::UserRenderConfig;

// the UI contains only visible elements. I.e only things are to be rendered.
// i.e. if shape is hidden - it's not in the UI. Treat it like intermediate datastructure
// between game state and vertex information that is passed in shader
struct UI {
    board: Board,
    mouse: MousePosition,
    score: Score,
}

struct Score {
    value: i16,
}

struct MousePosition {
    xy: XY,
}

// pixel coordinates.
#[derive(Debug, Default, Clone)]
pub struct XY(pub f32, pub f32);
impl XY {
    pub fn apply_offset(&self, offset: &OffsetXY) -> XY {
        XY(self.0 + (offset.0 as f32), self.1 + (offset.1 as f32))
    }
}
#[derive(Clone, Debug)]
pub struct OffsetXY(pub i16, pub i16);

// cell coordinate on the board, i.e. row, col pair.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct CellCoord {
    pub col: i16,
    pub row: i16,
}

impl CellCoord {
    pub fn new(col: i16, row: i16) -> Self {
        Self { col, row }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct Edge(pub u32, pub u32); // Edge is a pair of vertex indices
impl Edge {
    pub fn around_cell(coord: &CellCoord, board_size: usize) -> [Edge; 4] {
        let ix = cell_to_ix_4(coord, board_size);
        [
            Edge(ix[0], ix[1]).canonical(),
            Edge(ix[1], ix[2]).canonical(),
            Edge(ix[2], ix[3]).canonical(),
            Edge(ix[3], ix[0]).canonical(),
        ]
    }

    fn canonical(self) -> Edge {
        if self.0 < self.1 {
            self
        } else {
            Edge(self.1, self.0)
        }
    }
}

pub fn to_cell_space(top_left: XY, cell_size: f32, coord: &XY) -> CellCoord {
    let col = (coord.0 - top_left.0) / cell_size;
    let row = (coord.1 - top_left.1) / cell_size;

    return CellCoord::new(col.floor() as i16, row.floor() as i16);
}

//shapes -> index_buffer
pub fn render_panel(panel: &Panel, panel_width_cols: usize, board_index_offset: usize) -> Vec<u32> {
    let visible_cells: Vec<CellCoord> = panel
        .shapes_in_cell_space
        .iter()
        .filter_map(|(coord, &shape_index)| {
            panel
                .shape_choice
                .get(shape_index)
                .filter(|shape| shape.state == VISIBLE)
                .map(|_| coord.clone())
        })
        .collect();

    // convert grid + dimensions to indices for triangles
    return to_index_space(visible_cells, panel_width_cols, board_index_offset as u32);
}

/*
 offset represents the number of first vertex index.
 For example, if we store board and panel in the same vertex buffer, in order to compute panel indices, we need to consider that fact, that the first panel index
 is the max_board_index + 1. This is expressed by offset.
*/
pub fn to_index_space(cells: Vec<CellCoord>, max_col: usize, offset: u32) -> Vec<u32> {
    cells
        .iter()
        .flat_map(|cell_coord| cell_to_ix(cell_coord, max_col))
        .map(|i| i + offset)
        .collect()
}

fn cell_to_ix(coord: &CellCoord, max_col: usize) -> [u32; 6] {
    assert!(
        coord.row >= 0 && coord.col >= 0,
        "cell coordinate is negative: {:?}",
        coord
    );
    let row = coord.row as u32;
    let col = coord.col as u32;
    let stride = max_col as u32 + 1;

    let top_left = row * stride + col;
    let top_right = top_left + 1;
    let bottom_left = top_left + stride;
    let bottom_right = bottom_left + 1;
    return [
        top_left,
        bottom_left,
        bottom_right, // First triangle
        top_left,
        bottom_right,
        top_right, // Second triangle
    ];
}

pub fn cell_to_ix_4(coord: &CellCoord, max_col: usize) -> [u32; 4] {
    assert!(
        coord.row >= 0 && coord.col >= 0,
        "cell coordinate is negative: {:?}",
        coord
    );
    let row = coord.row;
    let col = coord.col;
    let stride = (max_col + 1) as i16;

    let top_left = row * stride + col;
    let top_right = top_left + 1;
    let bottom_left = top_left + stride;
    let bottom_right = bottom_left + 1;
    return [
        top_left as u32,
        top_right as u32,
        bottom_right as u32,
        bottom_left as u32,
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
                indices.extend(cell_to_ix(
                    &CellCoord::new(col as i16, row as i16),
                    board.size,
                ));
            }
        }
    }

    indices
}

pub fn within_bounds(px: f32, py: f32, x_max: f32, y_max: f32) -> bool {
    px >= 0.0 && px < x_max && py >= 0.0 && py < y_max
}

pub fn over_board(position: &XY, cfg: &UserRenderConfig) -> bool {
    let mouse_in_board_basis = position.apply_offset(&OffsetXY(
        -cfg.board_offset_x_px as i16,
        -cfg.board_offset_y_px as i16,
    ));
    let board_max = cfg.board_size_cols as f32 * cfg.cell_size_px;
    return within_bounds(
        mouse_in_board_basis.0,
        mouse_in_board_basis.1,
        board_max,
        board_max,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_cell() {
        let cells = vec![CellCoord::new(0, 0)]; // Top-left corner
        let indices = to_index_space(cells, 7);

        assert_eq!(
            indices,
            vec![
                0, 8, 9, // First triangle
                0, 9, 1, // Second triangle
            ]
        );
    }

    #[test]
    fn test_two_adjacent_cells_horizontally() {
        let cells = vec![CellCoord::new(0, 0), CellCoord::new(1, 0)]; // Two side-by-side cells in row 0
        let indices = to_index_space(cells, 7);

        assert_eq!(
            indices,
            vec![
                0, 8, 9, 0, 9, 1, // First cell
                1, 9, 10, 1, 10, 2, // Second cell
            ]
        );
    }

    #[test]
    fn test_two_adjacent_cells_vertically() {
        let cells = vec![CellCoord::new(0, 0), CellCoord::new(0, 1)]; // Two stacked cells
        let indices = to_index_space(cells, 7);

        assert_eq!(
            indices,
            vec![
                0, 8, 9, 0, 9, 1, // First cell
                8, 16, 17, 8, 17, 9, // Second cell (below first one)
            ]
        );
    }

    #[test]
    fn test_non_contiguous_cells_in_elonagated_grid() {
        let cells = vec![
            CellCoord::new(0, 0),
            CellCoord::new(2, 1),
            CellCoord::new(5, 2),
        ]; // Scattered cells
        let indices = to_index_space(cells, 7);

        assert_eq!(
            indices,
            vec![
                0, 8, 9, 0, 9, 1, // First cell (0,0)
                10, 18, 19, 10, 19, 11, // Second cell (2,1)
                21, 29, 30, 21, 30, 22, // Third cell (5,2)
            ]
        );
    }
}
