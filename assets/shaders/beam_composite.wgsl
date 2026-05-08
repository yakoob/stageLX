#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var beam_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var beam_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(beam_texture, beam_sampler, in.uv);
    // The half-res texture already has additive-blended beam color in RGB.
    // Output alpha = 1.0 so the quad's AlphaMode::Add passes RGB through unchanged.
    return vec4<f32>(sampled.rgb, 1.0);
}
