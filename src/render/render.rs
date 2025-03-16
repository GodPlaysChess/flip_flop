use std::cmp::max;
use std::collections::HashMap;
use std::iter;
use bytemuck::cast_slice;
use wgpu::{MemoryHints, PipelineLayout, ShaderModule, SurfaceConfiguration, TextureFormat, TextureUsages};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

// use wgpu_glyph::{ab_glyph, Section, Text};
use crate::game_entities::{Board, BOARD_SIZE, Cell, GameState, Shape};
use crate::render::vertex::{CursorState, generate_board_vertices, generate_panel_vertices, normalize_screen_to_ndc, Vertex};
use crate::render::space_converters::{render_board, render_panel};


const FONT_BYTES: &[u8] = include_bytes!("../../res/DejaVuSans.ttf");

pub struct Render<'a> {
    pub surface: wgpu::Surface<'a>,
    surface_config: SurfaceConfiguration,

    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    point_render_pipeline: wgpu::RenderPipeline,
    triangle_render_pipeline: wgpu::RenderPipeline,

    board_vertex_buffer: wgpu::Buffer,
    panel_vertex_buffer: wgpu::Buffer,
    cursor_vertex_buffer: wgpu::Buffer,

    board_index_buffer: wgpu::Buffer,
    panel_index_buffer: wgpu::Buffer,

    user_render_config: UserRenderConfig,

    // glyph_brush: wgpu_glyph::GlyphBrush<()>,
    // staging_belt: wgpu::util::StagingBelt,
}

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
        Self::new(
            12,
            5,
            10,
            40.0,
            30.0,
            100.0,
            100.0,
            100.0,
        )
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
    ) -> Self {
        let window_size = PhysicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT);
        let panel_offset_y_px = board_offset_y_px + cell_size_px * panel_cols as f32;

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

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();


        let (device, queue) = adapter.request_device(
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
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats
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


        let vertex_shader_module = device.create_shader_module(wgpu::include_wgsl!("../../res/shaders/textured.vert.wgsl"));
        let fragment_shader_module = device.create_shader_module(wgpu::include_wgsl!("../../res/shaders/textured.frag.wgsl"));

        let point_render_pipeline = create_pipeline(&device, &render_pipeline_layout, &vertex_shader_module, &fragment_shader_module, surface_config.format.clone(), wgpu::PrimitiveTopology::PointList);
        let triangle_render_pipeline = create_pipeline(&device, &render_pipeline_layout, &vertex_shader_module, &fragment_shader_module, surface_config.format.clone(), wgpu::PrimitiveTopology::TriangleList);

        let board_vertices = normalize_screen_to_ndc(generate_board_vertices(&render_config), render_config.window_size);
        let panel_vertices = normalize_screen_to_ndc(generate_panel_vertices(&render_config), render_config.window_size);


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
        let board_index_buffer = create_index_buffer(&device, render_config.board_size_cols * render_config.board_size_cols * 6);
        // there will be at most 4 shapes, 5 cells each, so we could limit it to 20 * 6
        let panel_index_buffer = create_index_buffer(&device, 120);

        surface.configure(&device, &surface_config);


        // let font = ab_glyph::FontArc::try_from_slice(FONT_BYTES).unwrap();
        // let glyph_brush =
        //     wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&device, config.format);
        // let staging_belt = wgpu::util::StagingBelt::new(1024);


        Self {
            surface,
            adapter,
            device,
            queue,
            surface_config,
            point_render_pipeline,
            triangle_render_pipeline,
            board_vertex_buffer,
            panel_vertex_buffer,
            cursor_vertex_buffer,
            board_index_buffer,
            panel_index_buffer,
            user_render_config: render_config,
            // glyph_brush,
            // staging_belt,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn render_state(&mut self, state: &GameState) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        //todo add cursor shadow
        let board_indices = render_board(&state.board);
        let panel_indices = render_panel(&state.shape_choice, self.user_render_config.panel_cols);

        match self.surface.get_current_texture() {
            Ok(frame) => {
                let board_vertex_number = (self.user_render_config.board_size_cols + 1) * (self.user_render_config.board_size_cols + 1);
                let panel_vertex_number = (self.user_render_config.panel_cols + 1) * (self.user_render_config.panel_rows + 1);
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
                render_pass.set_push_constants(wgpu::ShaderStages::FRAGMENT, 0, cast_slice(&[CursorState::NotACursor as u32]));

                render_pass.set_vertex_buffer(0, self.board_vertex_buffer.slice(..));
                render_pass.draw(0..board_vertex_number as u32, 0..1); // draw just indices

                render_pass.set_vertex_buffer(0, self.panel_vertex_buffer.slice(..));
                render_pass.draw(0..panel_vertex_number as u32, 0..1);

                // DRAW cells: board and panel
                render_pass.set_pipeline(&self.triangle_render_pipeline);


                //todo If board changed
                render_pass.set_vertex_buffer(0, self.board_vertex_buffer.slice(..));
                self.queue.write_buffer(&self.board_index_buffer, 0, cast_slice(&board_indices));
                render_pass.set_index_buffer(self.board_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..board_indices.len() as u32, 0, 0..1);

                // ✅ Bind the panel buffers ONCE, then draw all panel elements
                // render_pass.set_vertex_buffer(0, self.panel_vertex_buffer.slice(..));
                //todo If panel changed
                render_pass.set_vertex_buffer(0, self.panel_vertex_buffer.slice(..));
                self.queue.write_buffer(&self.panel_index_buffer, 0, cast_slice(&panel_indices));
                render_pass.set_index_buffer(self.panel_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..panel_indices.len() as u32, 0, 0..2);

                // ✅ Cursor changes every frame, so we must update the buffer
                let new_cursor_vertices = render_cursor(state.mouse_position, &self.user_render_config.cursor_size);
                self.queue.write_buffer(&self.cursor_vertex_buffer, 0, bytemuck::cast_slice(&new_cursor_vertices));
                render_pass.set_vertex_buffer(0, self.cursor_vertex_buffer.slice(..));

                render_pass.set_push_constants(wgpu::ShaderStages::FRAGMENT, 0, cast_slice(&[CursorState::Cursor as u32]));


                render_pass.draw(0..4, 0..1); // No index buffer needed, just 4 vertices

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

fn render_cursor(mouse_pos: (usize, usize), cursor_size: &f32) -> [Vertex; 4] {
    let mouse_x = mouse_pos.0 as f32;
    let mouse_y = mouse_pos.1 as f32;
    let half_size = cursor_size / 2.0;

    [
        Vertex::new(mouse_x - half_size, mouse_y - half_size), // Top-left
        Vertex::new(mouse_x + half_size, mouse_y - half_size), // Top-right
        Vertex::new(mouse_x + half_size, mouse_y + half_size), // Bottom-right
        Vertex::new(mouse_x - half_size, mouse_y + half_size), // Bottom-left
    ]
}

fn create_cursor_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Cursor Vertex Buffer"),
        size: (std::mem::size_of::<Vertex>() * 4) as wgpu::BufferAddress,
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

fn create_pipeline(device: &wgpu::Device,
                   render_pipeline_layout: &PipelineLayout,
                   vertex_shader_module: &ShaderModule,
                   fragment_shader_module: &ShaderModule,
                   format: TextureFormat,
                   topology: wgpu::PrimitiveTopology) -> wgpu::RenderPipeline {
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
            count: 1, // 2.
            mask: !0, // 3.
            alpha_to_coverage_enabled: false, // 4.
        },
        multiview: None, // 5.
        cache: None, // 6.
    })
}
