use std::collections::{HashMap, HashSet};
use std::iter;
use std::ops::Deref;
use std::rc::Rc;

use bytemuck::cast_slice;
use glyphon::Resolution;
use wgpu::core::id::markers::RenderPipeline;
use wgpu::util::DeviceExt;
use wgpu::{
    BufferAddress, MemoryHints, PipelineLayout, ShaderModule, SurfaceConfiguration, TextureFormat,
    TextureUsages,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::game_entities::{GameState, SelectedShape};
use crate::input::Input;
use crate::render::text_system::TextSystem;
use crate::render::vertex::{
    generate_board_vertices, generate_panel_vertices, normalize_screen_to_ndc, CursorState, Vertex,
};
use crate::space_converters::{
    over_board, render_board, render_panel, to_cell_space, within_bounds, CellCoord, Edge, XY,
};

const FONT_BYTES: &[u8] = include_bytes!("../../res/DejaVuSans.ttf");

#[derive(Clone)]
pub struct UserRenderConfig {
    pub window_size: PhysicalSize<u32>,
    // game cell space settings
    pub panel_cols: usize,
    pub panel_rows: usize,
    pub board_size_cols: usize,

    // pixel space settings
    pub cursor_size: f32,
    pub cell_size_px: f32,
    pub board_offset_x_px: f32,
    pub board_offset_y_px: f32,
    pub panel_offset_x_px: f32,
    pub panel_offset_y_px: f32,
}
const SCREEN_WIDTH: u32 = 1200;
const SCREEN_HEIGHT: u32 = 800;

impl Default for UserRenderConfig {
    fn default() -> Self {
        Self::new(12, 5, 10, 10.0, 30.0, 100.0, 100.0, 100.0, 100.0)
    }
}

impl UserRenderConfig {
    pub fn new(
        panel_cols: usize,
        panel_rows: usize,
        board_size: usize,
        cursor_size: f32,
        cell_size_px: f32,
        board_offset_x_px: f32,
        board_offset_y_px: f32,
        panel_offset_x_px: f32,
        board_panel_y_px: f32,
    ) -> Self {
        let window_size = PhysicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT);
        let panel_offset_y_px =
            board_offset_y_px + board_panel_y_px + cell_size_px * board_size as f32;

        Self {
            window_size,
            panel_cols,
            panel_rows,
            board_size_cols: board_size,
            cursor_size,
            cell_size_px,
            board_offset_x_px,
            board_offset_y_px,
            panel_offset_x_px,
            panel_offset_y_px, // Correctly computed here
        }
    }
}

pub struct Render<'a> {
    pub surface: wgpu::Surface<'a>,
    surface_config: SurfaceConfiguration,

    adapter: wgpu::Adapter,
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    point_render_pipeline: wgpu::RenderPipeline,
    triangle_render_pipeline: wgpu::RenderPipeline,
    contour_pipeline: wgpu::RenderPipeline,

    board_vertex_buffer: wgpu::Buffer,
    panel_vertex_buffer: wgpu::Buffer,
    cursor_vertex_buffer: wgpu::Buffer,

    board_index_buffer: wgpu::Buffer,
    panel_index_buffer: wgpu::Buffer,
    contour_index_buffer: wgpu::Buffer,

    user_render_config: UserRenderConfig,
    text_system: TextSystem,
}

impl<'a> Render<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window, render_config: UserRenderConfig) -> Render<'a> {
        println!("Vertex struct size: {}", Vertex::SIZE);

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::VULKAN, // VULKAN
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::PUSH_CONSTANTS,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits {
                            max_push_constant_size: 128,
                            ..wgpu::Limits::downlevel_webgl2_defaults()
                        }
                    } else {
                        wgpu::Limits {
                            max_push_constant_size: 128,
                            ..Default::default()
                        }
                    },
                    label: None,
                    memory_hints: MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let scale_factor = window.scale_factor(); // Get DPI scale
        let physical_width = (render_config.window_size.width as f64 * scale_factor) as u32;
        let physical_height = (render_config.window_size.height as f64 * scale_factor) as u32;

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: physical_width,
            height: physical_height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Triangle render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: 0..4,
                }],
            });

        let vertex_shader_module = device
            .create_shader_module(wgpu::include_wgsl!("../../res/shaders/textured.vert.wgsl"));
        let fragment_shader_module = device
            .create_shader_module(wgpu::include_wgsl!("../../res/shaders/textured.frag.wgsl"));

        let point_render_pipeline = create_pipeline(
            &device,
            &render_pipeline_layout,
            &vertex_shader_module,
            &fragment_shader_module,
            surface_config.format.clone(),
            wgpu::PrimitiveTopology::PointList,
        );
        let triangle_render_pipeline = create_pipeline(
            &device,
            &render_pipeline_layout,
            &vertex_shader_module,
            &fragment_shader_module,
            surface_config.format.clone(),
            wgpu::PrimitiveTopology::TriangleList,
        );

        let contour_pipeline = create_pipeline(
            &device,
            &render_pipeline_layout,
            &vertex_shader_module,
            &fragment_shader_module,
            surface_config.format.clone(),
            wgpu::PrimitiveTopology::LineStrip,
        );

        let board_vertices = normalize_screen_to_ndc(
            generate_board_vertices(&render_config),
            render_config.window_size,
        );
        let panel_vertices = normalize_screen_to_ndc(
            generate_panel_vertices(&render_config),
            render_config.window_size,
        );

        let board_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Board Vertex Buffer"),
            contents: cast_slice(&board_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let panel_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Panel Vertex Buffer"),
            contents: cast_slice(&panel_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let cursor_vertex_buffer = create_cursor_buffer(&device);

        // for the all board to be filled
        let board_index_buffer = create_index_buffer(
            &device,
            render_config.board_size_cols * render_config.board_size_cols * 6,
        );
        // there will be at most 4 shapes, 5 cells each, so we could limit it to 20 * 6
        let panel_index_buffer = create_index_buffer(&device, 120);
        let contour_index_buffer = create_index_buffer(&device, 20);

        surface.configure(&device, &surface_config);
        let resolution = Resolution {
            width: physical_width,
            height: physical_width,
        };

        let device = Rc::new(device);
        let queue = Rc::new(queue);
        let text_system = TextSystem::new(
            device.clone(),
            queue.clone(),
            TextureFormat::Rgba8UnormSrgb,
            resolution,
        );

        Self {
            surface,
            adapter,
            device,
            queue,
            surface_config,
            point_render_pipeline,
            triangle_render_pipeline,
            contour_pipeline,
            board_vertex_buffer,
            panel_vertex_buffer,
            cursor_vertex_buffer,
            board_index_buffer,
            panel_index_buffer,
            contour_index_buffer,
            user_render_config: render_config,
            text_system,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn render_state(&mut self, state: &GameState, input: &Input) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        //todo add cursor shadow
        let board_indices = render_board(&state.board);
        let panel_indices = render_panel(&state.panel, self.user_render_config.panel_cols);

        let mut contour_indices: Vec<u32> = Vec::new();
        //
        if let Some(selected_shape) = &state.selected_shape {
            if (over_board(&input.mouse_position, &self.user_render_config)) {
                contour_indices = render_contour(
                    &selected_shape,
                    &input.mouse_position,
                    &self.user_render_config,
                );
            };
        }

        match self.surface.get_current_texture() {
            Ok(frame) => {
                let board_vertex_number = (self.user_render_config.board_size_cols + 1)
                    * (self.user_render_config.board_size_cols + 1);
                let panel_vertex_number = (self.user_render_config.panel_cols + 1)
                    * (self.user_render_config.panel_rows + 1);
                let view = frame.texture.create_view(&Default::default());
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations::default(),
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // DRAW GRID
                render_pass.set_pipeline(&self.point_render_pipeline);
                render_pass.set_push_constants(
                    wgpu::ShaderStages::FRAGMENT,
                    0,
                    cast_slice(&[CursorState::NotACursor as u32]),
                );

                render_pass.set_vertex_buffer(0, self.board_vertex_buffer.slice(..));
                render_pass.draw(0..board_vertex_number as u32, 0..1); // draw just indices

                render_pass.set_vertex_buffer(0, self.panel_vertex_buffer.slice(..));
                render_pass.draw(0..panel_vertex_number as u32, 0..1);

                // DRAW cells: board and panel
                render_pass.set_pipeline(&self.triangle_render_pipeline);

                //todo If board changed
                render_pass.set_vertex_buffer(0, self.board_vertex_buffer.slice(..));
                self.queue
                    .write_buffer(&self.board_index_buffer, 0, cast_slice(&board_indices));
                render_pass
                    .set_index_buffer(self.board_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..board_indices.len() as u32, 0, 0..1);

                // ✅ Bind the panel buffers ONCE, then draw all panel elements
                //todo If panel changed
                render_pass.set_vertex_buffer(0, self.panel_vertex_buffer.slice(..));
                self.queue
                    .write_buffer(&self.panel_index_buffer, 0, cast_slice(&panel_indices));
                render_pass
                    .set_index_buffer(self.panel_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..panel_indices.len() as u32, 0, 0..2);

                // ✅ Cursor changes every frame, so we must update the buffer.
                // first we draw cursor as shape
                let mut cursor_offset_len: u32 = 0;
                let mut cursor_offset_bytes: usize = 0;
                if let Some(selected_shape) = &state.selected_shape {
                    // based on input, and selected shape, we can compute if it is over the board
                    if (over_board(&input.mouse_position, &self.user_render_config)) {
                        // can also choose to do it in the system, and just render here.
                        self.queue.write_buffer(
                            &self.contour_index_buffer,
                            0,
                            cast_slice(&contour_indices),
                        );
                        render_pass.set_pipeline(&self.contour_pipeline);
                        render_pass.set_vertex_buffer(0, self.board_vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            self.contour_index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..contour_indices.len() as u32, 0, 0..1);
                    }

                    render_pass.set_pipeline(&self.triangle_render_pipeline);
                    let cursor_shape_vertices = render_cursor_shape(
                        &input.mouse_position,
                        selected_shape,
                        self.user_render_config.cell_size_px,
                        &self.user_render_config.window_size,
                    );
                    cursor_offset_len = cursor_shape_vertices.len() as u32;
                    cursor_offset_bytes = cursor_shape_vertices.len() * size_of::<Vertex>();
                    // println!("Cursor selected vertices are {:?}", cursor_shape_vertices);
                    self.queue.write_buffer(
                        &self.cursor_vertex_buffer,
                        0,
                        cast_slice(&cursor_shape_vertices),
                    );
                }
                // then we draw the cursor
                let new_cursor_vertices = render_cursor(
                    &input.mouse_position,
                    &self.user_render_config.cursor_size,
                    &self.user_render_config.window_size,
                );
                self.queue.write_buffer(
                    &self.cursor_vertex_buffer,
                    cursor_offset_bytes as BufferAddress,
                    cast_slice(&new_cursor_vertices),
                );
                render_pass.set_vertex_buffer(0, self.cursor_vertex_buffer.slice(..));

                render_pass.draw(0..cursor_offset_len, 0..1);
                render_pass.set_push_constants(
                    wgpu::ShaderStages::FRAGMENT,
                    0,
                    cast_slice(&[CursorState::Cursor as u32]),
                );
                render_pass.draw(cursor_offset_len..(6 + cursor_offset_len), 0..1);

                self.text_system.set_score_text(state.score);
                self.text_system.render_score(&mut render_pass);
                drop(render_pass);

                // self.staging_belt.finish();
                self.queue.submit(iter::once(encoder.finish()));
                frame.present();
            }
            Err(wgpu::SurfaceError::Outdated) => {
                log::info!("Outdated surface texture");
                self.surface.configure(&self.device, &self.surface_config);
            }
            Err(e) => {
                log::error!("Error: {}", e);
            }
        }
    }
}

fn render_contour(
    shape: &SelectedShape,
    mouse_position: &XY,
    render_config: &UserRenderConfig,
) -> Vec<u32> {
    let placement_xy_0 = mouse_position.apply_offset(&shape.anchor_offset);
    let placement_0_cell = to_cell_space(
        XY(
            render_config.board_offset_x_px,
            render_config.board_offset_y_px,
        ),
        render_config.cell_size_px,
        &placement_xy_0,
    );
    let mut visible_cells = Vec::new();
    for (dx, dy) in shape.shape_type.cells() {
        let nx = placement_0_cell.col.wrapping_add(dx as i16);
        let ny = placement_0_cell.row.wrapping_add(dy as i16);
        if nx >= 0
            && nx < render_config.board_size_cols as i16
            && ny >= 0
            && ny < render_config.board_size_cols as i16
        {
            visible_cells.push(CellCoord::new(nx, ny));
        }
    }
    let mut edge_set: HashSet<Edge> = HashSet::new();

    for cell in &visible_cells {
        let edges = Edge::around_cell(cell, render_config.board_size_cols);
        for edge in &edges {
            if !edge_set.insert(*edge) {
                edge_set.remove(edge);
            }
        }
    }
    let contour_edges: Vec<Edge> = edge_set.into_iter().collect();

    order_edges_for_linestrip(contour_edges)
}

fn order_edges_for_linestrip(edges: Vec<Edge>) -> Vec<u32> {
    let mut ordered_vertices = Vec::new();
    let mut visited = HashSet::new();
    let mut edge_map: HashMap<u32, Vec<u32>> = HashMap::new();

    // Build adjacency map
    for edge in &edges {
        edge_map.entry(edge.0).or_insert_with(Vec::new).push(edge.1);
        edge_map.entry(edge.1).or_insert_with(Vec::new).push(edge.0);
    }

    // Start from any edge
    let first = edges[0].0;
    let mut current = first;
    ordered_vertices.push(current);
    visited.insert(first);

    while let Some(neighbors) = edge_map.get(&current) {
        let next = neighbors
            .iter()
            .filter(|&&n| !visited.contains(&n)) // Avoid revisiting
            .min(); // Pick the smallest to enforce order

        if let Some(&next) = next {
            ordered_vertices.push(next);
            visited.insert(next);
            current = next;
        } else {
            if (neighbors.contains(&first)) {
                ordered_vertices.push(first);
            }
            break;
        }
    }

    ordered_vertices
}

fn render_cursor(
    mouse_pos: &XY,
    cursor_size: &f32,
    physical_size: &PhysicalSize<u32>,
) -> [Vertex; 6] {
    let XY(mouse_x, mouse_y) = mouse_pos;
    let half_size = cursor_size / 2.0;

    let bot_left = Vertex::ndc_vertex(
        mouse_x - half_size,
        mouse_y - half_size,
        physical_size,
        true,
    );
    let bot_right = Vertex::ndc_vertex(
        mouse_x + half_size,
        mouse_y - half_size,
        physical_size,
        true,
    );
    let top_right = Vertex::ndc_vertex(
        mouse_x + half_size,
        mouse_y + half_size,
        physical_size,
        true,
    );
    let top_left = Vertex::ndc_vertex(
        mouse_x - half_size,
        mouse_y + half_size,
        physical_size,
        true,
    );
    [
        bot_right, bot_left, top_left, bot_right, top_left, top_right,
    ]
}

fn render_cursor_shape(
    mouse_pos: &XY,
    selected_shape: &SelectedShape,
    cell_size_px: f32,
    physical_size: &PhysicalSize<u32>,
) -> Vec<Vertex> {
    let zero = mouse_pos.apply_offset(&selected_shape.anchor_offset);
    let cells = selected_shape.shape_type.cells();

    let mut vertex_result: Vec<Vertex> = vec![];
    for cell in cells {
        let cell_x_offset = cell.0 as f32 * cell_size_px;
        let cell_y_offset = cell.1 as f32 * cell_size_px;
        let top_left = Vertex::ndc_vertex(
            zero.0 + cell_x_offset,
            zero.1 + cell_y_offset,
            physical_size,
            true,
        );
        let bot_left = Vertex::ndc_vertex(
            zero.0 + cell_x_offset,
            zero.1 + cell_size_px + cell_y_offset,
            physical_size,
            true,
        );
        let bot_right = Vertex::ndc_vertex(
            zero.0 + cell_size_px + cell_x_offset,
            zero.1 + cell_size_px + cell_y_offset,
            physical_size,
            true,
        );
        let top_right = Vertex::ndc_vertex(
            zero.0 + cell_size_px + cell_x_offset,
            zero.1 + cell_y_offset,
            physical_size,
            true,
        );
        vertex_result.extend(&[
            bot_left, bot_right, top_left, top_left, bot_right, top_right,
        ])
    }
    vertex_result
}

fn create_cursor_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Cursor Vertex Buffer"),
        // 6 vertices because of quad. If switch to index rendering - could keep it as 4
        //todo, currently we use the same buffer to render cursor shape. Could change it in the future.
        size: (size_of::<Vertex>() * 6 * 5) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // COPY_DST so we can update it
        mapped_at_creation: false,
    })
}

fn create_index_buffer(device: &wgpu::Device, max_indices: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Dynamic Index Buffer"),
        size: (size_of::<u32>() * max_indices) as wgpu::BufferAddress, // Preallocate space
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST, // COPY_DST allows updates
        mapped_at_creation: false,
    })
}

fn create_pipeline(
    device: &wgpu::Device,
    render_pipeline_layout: &PipelineLayout,
    vertex_shader_module: &ShaderModule,
    fragment_shader_module: &ShaderModule,
    format: TextureFormat,
    topology: wgpu::PrimitiveTopology,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader_module,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::DESC],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),

        primitive: wgpu::PrimitiveState {
            topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,                         // 2.
            mask: !0,                         // 3.
            alpha_to_coverage_enabled: false, // 4.
        },
        multiview: None, // 5.
        cache: None,     // 6.
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_entities::ShapeType;
    use crate::space_converters::OffsetXY;

    fn mock_render_config() -> UserRenderConfig {
        UserRenderConfig {
            window_size: Default::default(),
            panel_cols: 0,
            board_offset_x_px: 0.0,
            board_offset_y_px: 0.0,
            panel_offset_x_px: 0.0,
            cell_size_px: 10.0,
            board_size_cols: 10,
            panel_rows: 0,
            cursor_size: 0.0,
            panel_offset_y_px: 0.0,
        }
    }

    #[test]
    fn test_render_contour_single_cell() {
        let shape = SelectedShape {
            shape_type: ShapeType::O,
            anchor_offset: OffsetXY(0, 0),
        }; // 1x1 shape
        let mouse_position = XY(15.0, 15.0);
        let render_config = mock_render_config();

        let contour = render_contour(&shape, &mouse_position, &render_config);

        assert_eq!(
            contour.len(),
            5,
            "A single cell should have 4 contour edges"
        );
    }

    #[test]
    fn test_render_contour_l_shape() {
        let shape = SelectedShape {
            shape_type: ShapeType::L1,
            anchor_offset: OffsetXY(0, 0),
        }; // L-shape
        let mouse_position = XY(15.0, 15.0);
        let render_config = mock_render_config();

        let contour = render_contour(&shape, &mouse_position, &render_config);
        print!("contour {:?}", contour);

        assert_eq!(
            contour.len(),
            11,
            "L-shape should have a valid contour with correct edges"
        );
    }

    #[test]
    fn test_order_edges_for_linestrip() {
        let edges = vec![
            Edge(1, 2),
            Edge(2, 3),
            Edge(3, 4),
            Edge(4, 1), // Forms a square loop
        ];

        let ordered = order_edges_for_linestrip(edges);

        assert_eq!(
            ordered.len(),
            5,
            "Should return a closed loop with one duplicate start"
        );
        assert_eq!(ordered[0], ordered[4], "Last vertex should match first");
    }

    #[test]
    fn test_order_edges_for_linestrip_incomplete_loop() {
        let edges = vec![
            Edge(1, 2),
            Edge(2, 3),
            Edge(3, 4), // Open path, no closure
        ];

        let ordered = order_edges_for_linestrip(edges);

        assert_eq!(
            ordered.len(),
            4,
            "Should return an ordered path with no duplicate end"
        );
    }
}
