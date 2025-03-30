// --- GRID VERTEX SHADER ---
@group(0) @binding(0) var my_texture: texture_2d<f32>;
@group(0) @binding(1) var my_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_grid(@location(0) pos: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.tex_coords = (pos + vec2<f32>(1.0, 1.0)) * 0.5; // Convert from NDC (-1 to 1) to UV (0 to 1)
    return out;
}

// --- GRID FRAGMENT SHADER ---
@fragment
fn fs_grid(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(my_texture, my_sampler, in.tex_coords);
}