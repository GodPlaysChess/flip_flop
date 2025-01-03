use rusttype::{Font, point, Scale};
use crate::game_entities::{BOARD_SIZE, Cell, CELL_SIZE, GameState, Shape, ShapeType};
use crate::{HEIGHT, WIDTH};
use crate::events::{Event, XY};
use crate::events::Event::{BoardUpdated, ScoreUpdated, ShapeChoiceUpdate};
use crate::game_entities::ShapeState::VISIBLE;
use crate::logic::{SHAPE_LINE_COORD_X, SHAPE_LINE_COORD_Y};

const RED: u32 = 0xff0000;
const GREEN: u32 = 0x00ff00;
const WHITE: u32 = 0xffffff;
pub const BLACK: u32 = 0x000000;

// Pre-render Cache: Cache rendered characters into a texture atlas for faster re-use.
// GPU Acceleration: Use a rendering library like wgpu or pixels for hardware-accelerated text rendering if performance becomes critical.
pub struct Renderer {
    pub width: usize, // borders of the screen
    pub height: usize,
}

impl Renderer {
    pub fn draw_text(&self, text: &str, x: usize, y: usize, font_data: &[u8], font_size: f32, color: u32, buffer: &mut Vec<u32>) {
        let font = Font::try_from_bytes(font_data).expect("Error constructing font");
        let scale = Scale::uniform(font_size);

        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);

        for glyph in font.layout(text, scale, offset) {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = x as i32 + bounding_box.min.x + gx as i32;
                    let py = y as i32 + bounding_box.min.y + gy as i32;

                    if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                        let idx = (py as usize * self.width + px as usize) as usize;
                        buffer[idx] = (color as f32 * v) as u32;
                    }
                });
            }
        }
    }
}

pub fn draw_cursor(selected_shape: &Option<ShapeType>, mouse_position: (usize, usize), buffer: &mut Vec<u32>) {
    if let Some(kind) = selected_shape {
        let (mx, my) = mouse_position;
        // todo properly draw around cursor
        draw_shape_kind(kind, mx, my, GREEN, buffer);
    }
}

pub fn draw_mouse_click(x_y: (usize, usize), buffer: &mut Vec<u32>) {
    for i in 0..10 {
        for j in 0..10 {
            let y = x_y.1 + i;
            let x = x_y.0 + j;
            buffer[(y * WIDTH + x) % (WIDTH * HEIGHT)] = GREEN
        }
    }
}

pub fn draw_score(score: u32, renderer: &Renderer, font_data: &[u8], buffer: &mut Vec<u32>) {
    renderer.draw_text(score.to_string().as_str(), 800, 20, font_data, 36.0, WHITE, buffer);
}

pub fn draw_board(game: &GameState, buffer: &mut Vec<u32>) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            draw_cell(x, y, &game.board.grid[y][x], buffer);
        }
    }
}

pub fn draw_highlight(kind: &ShapeType, mouse_position: XY, valid: bool, buffer: &mut Vec<u32>) {
    // Highlight placement zone if a shape is selected
    //todo duplication here and inside game object
    let XY(x, y) = mouse_position;
    let n = x / CELL_SIZE;
    let m = y / CELL_SIZE;

    let color = if valid { GREEN } else { RED };
    for (dx, dy) in Shape::cells(kind) {
        let nx = n.wrapping_add(dx);
        let ny = m.wrapping_add(dy);
        if nx < BOARD_SIZE && ny < BOARD_SIZE {
            draw_colored_cell(nx, ny, color, buffer);
        }
    }
}

pub fn draw_shape_kind(shape: &ShapeType, pos_x: usize, pos_y: usize, color: u32, buffer: &mut Vec<u32>) {
    for (x, y) in Shape::cells(shape) {
        let px = pos_x + x * CELL_SIZE;
        let py = pos_y + y * CELL_SIZE;
        draw_rect(px, py, CELL_SIZE, CELL_SIZE, color, buffer);
    }
}

fn draw_colored_cell(n: usize, m: usize, color: u32, buffer: &mut Vec<u32>) {
    let px = n * CELL_SIZE;
    let py = m * CELL_SIZE;
    draw_rect(px, py, CELL_SIZE, CELL_SIZE, color, buffer);
}

pub fn draw_cell(n: usize, m: usize, cell: &Cell, buffer: &mut Vec<u32>) {
    let color = match cell {
        Cell::Empty => 0x202020, // Dark gray for empty cells
        Cell::Filled => WHITE, // White for filled cells
    };
    draw_colored_cell(n, m, color, buffer);
}

pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: u32, buffer: &mut Vec<u32>) {
    for row in 0..height {
        for col in 0..width {
            let px = x + col;
            let py = y + row;
            if px < WIDTH && py < HEIGHT {
                buffer[py * WIDTH + px] = color;
            }
        }
    }
}

pub fn update_background(event: Event, game_state: &GameState, buffer: &mut Vec<u32>, renderer: &mut Renderer, font_data: &[u8]) {
    match event {
        ScoreUpdated(new_score) => {
            draw_score(new_score, renderer, font_data, buffer)
        }
        BoardUpdated(updates) => {
            for update in updates {
                draw_cell(update.coord.0, update.coord.1, &update.cell, buffer);
            }
        }
        // to simplify the deselecting etc, we just redraw the shapes below
        ShapeChoiceUpdate => {
            // println!("Updating background shape choice with shapes {:?}", game_state.shape_choice);
            draw_shape_choice(game_state, buffer);
        }
    }
}

pub fn draw_foreground(game_state: &GameState, buffer: &mut Vec<u32>) {
    draw_cursor(&game_state.selected_shape, game_state.mouse_position, buffer);
    if let Some(shape) = game_state.selected_shape {
        let valid_placement = game_state.is_valid_placement_of_selected_shape();
        draw_highlight(&shape, XY(game_state.mouse_position.0, game_state.mouse_position.1), valid_placement, buffer);
    }

    // if mouse is clicked
    draw_mouse_click(game_state.last_click_position, buffer);
}

pub fn draw_background(game_state: &GameState, buffer: &mut Vec<u32>, font_data: &[u8], renderer: &mut Renderer) {
    // if board is changed
    draw_board(game_state, buffer);
    draw_shape_choice(game_state, buffer);
    // if score is changed
    draw_score(game_state.score, renderer, font_data, buffer);
}

fn draw_shape_choice(game_state: &GameState, buffer: &mut Vec<u32>) {
    // todo hardcoded coordinates and width
    draw_rect(SHAPE_LINE_COORD_X, SHAPE_LINE_COORD_Y, 1000, 300, BLACK, buffer);
    for shape in &game_state.shape_choice {
        let pos_x = SHAPE_LINE_COORD_X + CELL_SIZE * shape.x_cell_coordinate;
        if shape.state == VISIBLE {
            draw_shape_kind(&shape.kind, pos_x, SHAPE_LINE_COORD_Y, RED, buffer);
        }
    }
}

fn render_cursor(cursor_position: (f64, f64), encoder: &mut wgpu::CommandEncoder, render_pipeline: &wgpu::RenderPipeline, render_target: &wgpu::TextureView) {
    // Convert the cursor's logical position to a format suitable for rendering.
    let cursor_x = cursor_position.0 as f32; // Adjust scaling if needed
    let cursor_y = cursor_position.1 as f32;

    let cursor_size = 10.0; // Size of the cursor (in pixels)

    // Here you'd update vertex buffers or shaders to render the cursor at (cursor_x, cursor_y)
    // For example, bind and draw a quad or circle.
}

