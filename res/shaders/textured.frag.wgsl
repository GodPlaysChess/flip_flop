//@fragment
//fn fs_main() -> @location(0) vec4<f32> {
//    return vec4<f32>(1.0, 0.0, 0.0, 1.0);  // Red color
//}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4(pos.xy * 0.5 + 0.5, 0.0, 1.0); // Visualize screen positions
}
