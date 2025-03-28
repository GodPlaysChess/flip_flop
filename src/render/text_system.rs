use std::rc::Rc;

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::{DepthStencilState, MultisampleState, RenderPass};

pub struct TextSystem {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    buffer: Buffer,
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    viewport: Viewport,
}

impl TextSystem {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        format: wgpu::TextureFormat,
        resolution: Resolution,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device.as_ref());
        let mut viewport = Viewport::new(&device, &cache);
        viewport.update(queue.as_ref(), resolution);
        let mut atlas = TextAtlas::new(device.as_ref(), queue.as_ref(), &cache, format);
        let renderer = TextRenderer::new(
            &mut atlas,
            device.as_ref(),
            MultisampleState::default(),
            Some(DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8, // Must match render pass
                depth_write_enabled: false,  // No depth writes for these pipelines
                depth_compare: wgpu::CompareFunction::Always, // Ignore depth testing
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState::IGNORE,
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
        );
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 40.0));
        buffer.set_size(&mut font_system, Some(200.0), Some(50.0));

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            buffer,
            device,
            queue,
            viewport,
        }
    }

    pub fn render_score(&mut self, render_pass: &mut RenderPass) {
        if self.buffer.lines.is_empty() {
            println!("⚠️ Warning: Buffer is empty! Skipping text rendering.");
            return;
        }
        // Prepare text
        let text_area = TextArea {
            buffer: &self.buffer,
            left: 1000.0, // X Position (left corner)
            top: 100.0,   // Y Position (top corner)
            scale: 1.0,
            bounds: TextBounds::default(),
            default_color: Color::rgba(0, 255, 0, 255),
            custom_glyphs: &[],
        };

        if let Err(e) = self.renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            vec![text_area],
            &mut self.swash_cache,
        ) {
            println!("❌ Error in renderer.prepare: {:?}", e);
        }

        self.renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }

    pub fn set_score_text(&mut self, score: u32) {
        self.buffer.set_text(
            &mut self.font_system,
            &format!("Score: {}", score),
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
    }
}
