use std::cmp::max;
use std::iter;
use wgpu::{MemoryHints, PipelineLayout, ShaderModule, SurfaceConfiguration, TextureFormat, TextureUsages};
use wgpu::util::DeviceExt;
use winit::window::Window;

// use wgpu_glyph::{ab_glyph, Section, Text};
use crate::game_entities::{Board, BOARD_SIZE, Cell, GameState, Shape};
use crate::render::buffer::{generate_board_vertices, generate_panel_vertices, normalize_screen_to_ndc, Vertex};

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

    // glyph_brush: wgpu_glyph::GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,
}

const FONT_BYTES: &[u8] = include_bytes!("../../res/DejaVuSans.ttf");

impl<'a> Render<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window, size: winit::dpi::PhysicalSize<u32>) -> Render<'a> {
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
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
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
        let physical_width = (size.width as f64 * scale_factor) as u32;
        let physical_height = (size.height as f64 * scale_factor) as u32;

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: physical_width,
            height: physical_height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };


        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let vertex_shader_module = device.create_shader_module(wgpu::include_wgsl!("../../res/shaders/textured.vert.wgsl"));
        let fragment_shader_module = device.create_shader_module(wgpu::include_wgsl!("../../res/shaders/textured.frag.wgsl"));

        let point_render_pipeline = create_pipeline(&device, &render_pipeline_layout, &vertex_shader_module, &fragment_shader_module, surface_config.format.clone(), wgpu::PrimitiveTopology::PointList);
        let triangle_render_pipeline = create_pipeline(&device, &render_pipeline_layout, &vertex_shader_module, &fragment_shader_module, surface_config.format.clone(), wgpu::PrimitiveTopology::TriangleList);

        let board_vertices = normalize_screen_to_ndc(generate_board_vertices(), size);
        let panel_vertices = normalize_screen_to_ndc(generate_panel_vertices(), size);

        let board_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Board Vertex Buffer"),
            contents: bytemuck::cast_slice(&board_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let panel_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Panel Vertex Buffer"),
            contents: bytemuck::cast_slice(&panel_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let cursor_vertex_buffer = create_cursor_buffer(&device);

        let board_index_buffer = create_index_buffer(&device, BOARD_SIZE * BOARD_SIZE * 6);
        let panel_index_buffer = create_index_buffer(&device, 14 * 6 * 2);

        surface.configure(&device, &surface_config);
        let size = surface.get_current_texture().unwrap().texture.size();
        println!("wgpu Render Target Size: {}x{}", size.width, size.height);

        // let font = ab_glyph::FontArc::try_from_slice(FONT_BYTES).unwrap();
        // let glyph_brush =
        //     wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&device, config.format);
        let staging_belt = wgpu::util::StagingBelt::new(1024);


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
            // glyph_brush,
            staging_belt,
        }
    }


    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
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
        let board_indicies = render_board(&state.board);
        // let board_indicies: Vec<u32> = vec![0, 13, 14/*, 0, 14, 1, 20, 33, 34, 20, 34, 21, 43, 56, 57, 43, 57, 44*/];
        // let board_indicies = vec![1u32, 112u32,120u32];

        let panel_indicies = render_panel(&state.shape_choice);

        match self.surface.get_current_texture() {
            Ok(frame) => {
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
                // ✅ Bind the board buffers ONCE, then draw all board elements
                render_pass.set_vertex_buffer(0, self.board_vertex_buffer.slice(..));

                // DRAW GRID
                render_pass.set_pipeline(&self.point_render_pipeline);
                render_pass.draw(0..(BOARD_SIZE*BOARD_SIZE) as u32 - 1, 0..1);

                // DRAW cells
                render_pass.set_pipeline(&self.triangle_render_pipeline);
                //todo If board changed
                // ✅ Upload new index buffer to GPU
                self.queue.write_buffer(&self.board_index_buffer, 0, bytemuck::cast_slice(&board_indicies));
                render_pass.set_index_buffer(self.board_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..board_indicies.len() as u32, 0, 0..1);

                // ✅ Bind the panel buffers ONCE, then draw all panel elements
                // render_pass.set_vertex_buffer(0, self.panel_vertex_buffer.slice(..));
                //todo If panel changed
                // self.queue.write_buffer(&self.panel_index_buffer, 0, bytemuck::cast_slice(&board_indicies));
                // render_pass.set_index_buffer(self.panel_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                // render_pass.draw_indexed(0..panel_indicies.len() as u32, 0, 0..1);

                // ✅ Cursor changes every frame, so we must update the buffer
                // let new_cursor_vertices = render_cursor(state.mouse_position);
                // self.queue.write_buffer(&self.cursor_vertex_buffer, 0, bytemuck::cast_slice(&new_cursor_vertices));
                // render_pass.set_vertex_buffer(0, self.cursor_vertex_buffer.slice(..));
                // render_pass.draw(0..4, 0..1); // No index buffer needed, just 4 vertices

                drop(render_pass);

                self.staging_belt.finish();
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

//todo draw panel
fn render_panel(shapes: &Vec<Shape>) -> Vec<u32> {
    let mut indices = Vec::new();
    // let board_size = board.grid.len();
    //
    // for y in 0..board_size {
    //     for x in 0..board_size {
    //         if let Cell::Filled = board.grid[y][x] {
    //             let top_left = (y * (board_size + 1) + x) as u32;
    //             let top_right = top_left + 1;
    //             let bottom_left = top_left + (board_size as u32 + 1);
    //             let bottom_right = bottom_left + 1;
    //
    //             // Two triangles per cell (diagonal split)
    //             indices.extend_from_slice(&[
    //                 top_left, bottom_left, bottom_right, // First triangle
    //                 top_left, bottom_right, top_right,  // Second triangle
    //             ]);
    //         }
    //     }
    // }

    indices
}

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


fn render_cursor(mouse_pos: (usize, usize)) -> [Vertex; 4] {
    let mouse_x = max(20, mouse_pos.0);
    let mouse_y = max(20, mouse_pos.1);
    let cursor_size = 40;
    let half_size = cursor_size / 2;

    [
        Vertex::from_uszie(mouse_x - half_size, mouse_y - half_size), // Top-left
        Vertex::from_uszie(mouse_x + half_size, mouse_y - half_size), // Top-right
        Vertex::from_uszie(mouse_x + half_size, mouse_y + half_size), // Bottom-right
        Vertex::from_uszie(mouse_x - half_size, mouse_y + half_size), // Bottom-left
    ]
}

fn create_cursor_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    let cursor_vertices = [
        Vertex::new(-5.0, -5.0), // Example cursor shape
        Vertex::new(5.0, -5.0),
        Vertex::new(5.0, 5.0),
        Vertex::new(-5.0, 5.0),
    ];

    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Cursor Vertex Buffer"),
        size: (std::mem::size_of::<Vertex>() * cursor_vertices.len()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // COPY_DST so we can update it
        mapped_at_creation: false,
    })
}

fn create_index_buffer(device: &wgpu::Device, max_indices: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Dynamic Index Buffer"),
        size: (std::mem::size_of::<u16>() * max_indices) as wgpu::BufferAddress, // Preallocate space
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
                format: format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),

        primitive: wgpu::PrimitiveState {
            topology: topology,
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
