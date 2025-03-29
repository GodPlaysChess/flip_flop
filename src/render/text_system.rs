use std::rc::Rc;

use crate::game_entities::GameStats;
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::{MultisampleState, RenderPass};

pub struct TextSystem {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    score_buffer: Buffer,
    target_score_buffer: Buffer,
    level_buffer: Buffer,
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
            None,
        );
        let mut score_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 40.0));
        let mut target_score_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 40.0));
        let mut level_buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 40.0));
        score_buffer.set_size(&mut font_system, Some(200.0), Some(50.0));
        target_score_buffer.set_size(&mut font_system, Some(200.0), Some(50.0));
        level_buffer.set_size(&mut font_system, Some(200.0), Some(50.0));

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            score_buffer,
            level_buffer,
            target_score_buffer,
            device,
            queue,
            viewport,
        }
    }

    pub fn render_score(&mut self, game_stats: &GameStats, render_pass: &mut RenderPass) {
        &self.score_buffer.set_text(
            &mut self.font_system,
            &format!("Score: {}", game_stats.current_score),
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
        let score_text = TextArea {
            buffer: &mut self.score_buffer,
            left: 800.0, // X Position (left corner)
            top: 100.0,   // Y Position (top corner)
            scale: 1.0,
            bounds: TextBounds::default(),
            default_color: Color::rgba(0, 255, 0, 255),
            custom_glyphs: &[],
        };

        &self.target_score_buffer.set_text(
            &mut self.font_system,
            &format!("Target: {}", game_stats.target_score),
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        let target_score_text = TextArea {
            buffer: &mut self.target_score_buffer,
            left: 800.0, // X Position (left corner)
            top: 200.0,   // Y Position (top corner)
            scale: 1.0,
            bounds: TextBounds::default(),
            default_color: Color::rgba(0, 255, 0, 255),
            custom_glyphs: &[],
        };

        &self.level_buffer.set_text(
            &mut self.font_system,
            &format!("Level: {}", game_stats.level),
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        let level_text = TextArea {
            buffer: &mut self.level_buffer,
            left: 500.0, // X Position (left corner)
            top: 25.0,   // Y Position (top corner)
            scale: 2.0,
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
            vec![score_text, target_score_text, level_text],
            &mut self.swash_cache,
        ) {
            println!("❌ Error in renderer.prepare: {:?}", e);
        }

        self.renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }

}
