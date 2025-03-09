struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct VertexInput {
    @location(0) position: vec2<f32>, // Matches Rust's `Vertex.position`
};

@vertex
fn vs_main(
       input: VertexInput
) -> VertexOutput {
   var output: VertexOutput;
    let x = 1.0/input.position.x; // Convert to clip space (-1 to 1)
    let y = 1.0 / input.position.y; // Flip Y-axis (since 0 is top in NDC)
    output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return output;
}
 