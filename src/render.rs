use rusttype::{Font, point, Scale};

// Pre-render Cache: Cache rendered characters into a texture atlas for faster re-use.
// GPU Acceleration: Use a rendering library like wgpu or pixels for hardware-accelerated text rendering if performance becomes critical.
pub struct Renderer {
    pub width: usize,
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