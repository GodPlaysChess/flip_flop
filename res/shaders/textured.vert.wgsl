struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct VertexInput {
    @location(0) position: vec2<f32>, // Matches Rust's `Vertex.position`
};

//@vertex
//fn vs_main(
//       input: VertexInput
//) -> VertexOutput {
//    var output: VertexOutput;
//    let x = input.position.x; // Convert to clip space (-1 to 1)
//    let y = input.position.y; // Flip Y-axis (since 0 is top in NDC)
//    output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
//    return output;
//}

// debug
@vertex
fn vs_main(
    input: VertexInput
) -> @builtin(position) vec4<f32> {
    let pos = input.position;
    return vec4<f32>(pos, 0.0, 1.0);
}
 