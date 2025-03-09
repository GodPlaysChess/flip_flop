pub const U32_SIZE: wgpu::BufferAddress = std::mem::size_of::<u32>() as wgpu::BufferAddress;
pub const CELL_SIZE: f32 = 50.0;

pub const BOARD_OFFSET_X: f32 = 100.0;
pub const BOARD_OFFSET_Y: f32 = 100.0;

pub const PANEL_OFFSET_X: f32 = 100.0;
pub const PANEL_OFFSET_Y: f32 = BOARD_OFFSET_Y + CELL_SIZE * 12.0;


#[derive(Copy, Clone)]
pub struct Vertex {
    #[allow(dead_code)]
    position: cgmath::Vector2<f32>,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub const SIZE: wgpu::BufferAddress = std::mem::size_of::<Self>() as wgpu::BufferAddress;
    pub const DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: Self::SIZE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2
        ],
    };

    pub fn new(x: f32, y: f32) -> Self {
        Self { position: (x, y).into() }
    }

    pub fn from_uszie(x: usize, y: usize) -> Self {
        Self { position: (x as f32, y as f32).into() }
    }
}

pub fn generate_board_vertices() -> Vec<Vertex> {
    let mut vertices = Vec::new();

    for row in 0..=10 {  // We need 11 rows (0-10) for 10 cells
        for col in 0..=10 {
            let x = col as f32 * CELL_SIZE + BOARD_OFFSET_X;
            let y = row as f32 * CELL_SIZE + BOARD_OFFSET_Y;
            vertices.push(Vertex::new(x, y));
        }
    }

    vertices
}

pub fn generate_panel_vertices() -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for row in 0..=5 {  // We need 11 rows (0-10) for 10 cells
        for col in 0..=12 {
            let x = col as f32 * CELL_SIZE + PANEL_OFFSET_X;
            let y = row as f32 * CELL_SIZE + PANEL_OFFSET_Y;
            vertices.push(Vertex::new(x, y));
        }
    }

    vertices
}

// pub struct StagingBuffer {
//     buffer: wgpu::Buffer,
//     size: wgpu::BufferAddress,
// }
//
// impl StagingBuffer {
//     pub fn new<T: bytemuck::Pod + Sized>(
//         device: &wgpu::Device,
//         data: &[T],
//         is_index_buffer: bool,
//     ) -> StagingBuffer {
//         StagingBuffer {
//             buffer: device.create_buffer_init(&BufferInitDescriptor {
//                 contents: bytemuck::cast_slice(data),
//                 usage: wgpu::BufferUsages::COPY_SRC
//                     | if is_index_buffer {
//                     wgpu::BufferUsages::INDEX
//                 } else {
//                     wgpu::BufferUsages::empty()
//                 },
//                 label: Some("Staging Buffer"),
//             }),
//             size: size_of_slice(data) as wgpu::BufferAddress,
//         }
//     }
//
//     pub fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, other: &wgpu::Buffer) {
//         encoder.copy_buffer_to_buffer(&self.buffer, 0, other, 0, self.size)
//     }
// }