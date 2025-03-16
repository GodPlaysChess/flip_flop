use winit::dpi::PhysicalSize;
use crate::render::render::UserRenderConfig;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    #[allow(dead_code)]
    pub position: cgmath::Vector2<f32>,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: Self::SIZE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
        ],
    };

    pub fn new(x: f32, y: f32) -> Self {
        Self { position: (x, y).into() }
    }

    pub fn from_uszie(x: usize, y: usize) -> Self {
        Self { position: (x as f32, y as f32).into() }
    }

    pub fn ndc_vertex(x: f32, y: f32, size: &PhysicalSize<u32>, clamped: bool) -> Self {
        let width = size.width as f32;
        let height = size.height as f32;
        let ndc_x = (x / width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / height) * 2.0; // Flip Y-axis
        if clamped {
            Self::new(ndc_x.max(-1.0).min(1.0), ndc_y.max(-1.0).min(1.0))
        } else {
            Self::new(ndc_x, ndc_y)

        }
    }

}



pub fn normalize_screen_to_ndc(v: Vec<Vertex>, size: PhysicalSize<u32>) -> Vec<Vertex> {
    v.into_iter()
        .map(|vertex| {
            Vertex::ndc_vertex(vertex.position.x, vertex.position.y, &size, false)
        })
        .collect()
}

pub fn generate_panel_vertices(user_render_config: &UserRenderConfig) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for row in 0..=user_render_config.panel_rows {
        for col in 0..=user_render_config.panel_cols {
            let x = col as f32 * user_render_config.cell_size_px + user_render_config.panel_offset_x_px;
            let y = row as f32 * user_render_config.cell_size_px + user_render_config.panel_offset_y_px;
            vertices.push(Vertex::new(x, y));
        }
    }
    println!("Generated {:?} panel vertices", vertices.len());
    vertices
}

pub fn generate_board_vertices(user_render_config: &UserRenderConfig) -> Vec<Vertex> {
    let mut vertices = Vec::new();

    for row in 0..=user_render_config.board_size_cols {
        for col in 0..=user_render_config.board_size_cols {
            let x = col as f32 * user_render_config.cell_size_px + user_render_config.board_offset_x_px;
            let y = row as f32 * user_render_config.cell_size_px + user_render_config.board_offset_y_px;
            vertices.push(Vertex::new(x, y));
        }
    }

    vertices
}

#[repr(u32)] // Ensures it's represented as a u32 in memory
#[derive(Clone, Copy, Debug)]
pub enum CursorState {
    NotACursor = 0,
    Cursor = 1,
}
