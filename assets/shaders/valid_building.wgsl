#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> valid: u32;

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    if valid == 0 {
        return vec4<f32>(1.0, 0.0, 0.0, 0.4);
    } else {
        return vec4<f32>(0.0, 1.0, 0.0, 0.4);
    }
}
