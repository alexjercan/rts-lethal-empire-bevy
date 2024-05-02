#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var nearest_sampler: sampler;
@group(2) @binding(2) var<storage, read> mapping: array<u32>;
@group(2) @binding(3) var<uniform> size: u32;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let coords = clamp(vec2<u32>(mesh.uv * f32(size)), vec2<u32>(0u), vec2<u32>(size - 1));
    let index = coords.y * size + coords.x;
    let texture_index = mapping[index];

    let inner_uv = fract(mesh.uv * f32(size));
    return textureSample(textures[texture_index], nearest_sampler, inner_uv);
}
