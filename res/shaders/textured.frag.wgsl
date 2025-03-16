struct PushConstants {
    is_cursor: u32
}
var<push_constant> c: PushConstants;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
//    let c: f32 = select(0.0, 1.0, is_cursor == 1u);
//    return vec4<f32>(1.0, c, 0.0, 1.0);
    if c.is_cursor == 1u {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0); // 🔴 Red for cursor
    } else {
        return vec4<f32>(0.5, 0.3, 0.0, 1.0); // 🟡 Yellowish for everything else
    }
}

