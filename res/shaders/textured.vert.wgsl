struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct VertexInput {
    @location(0) position: vec2<f32>, // Matches Rust's `Vertex.position`
};

// debug
@vertex
fn vs_main(
    input: VertexInput
) -> @builtin(position) vec4<f32> {
    let pos = input.position;
    return vec4<f32>(pos, 0.0, 1.0);
}
 