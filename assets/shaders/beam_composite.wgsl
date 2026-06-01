#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var beam_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var beam_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(beam_texture, beam_sampler, in.uv);
    // The half-res texture already holds additive-blended beam colour in RGB.
    // AlphaMode::Add is premultiplied-alpha blending: result = src.rgb + dst·(1−src.a).
    // Output alpha MUST be 0.0 so the destination (the lit 3-D scene behind this
    // fullscreen quad) is preserved and the beam colour is added on top. Using
    // alpha = 1.0 zeroes dst and erases the whole scene behind the quad.
    return vec4<f32>(sampled.rgb, 0.0);
}
