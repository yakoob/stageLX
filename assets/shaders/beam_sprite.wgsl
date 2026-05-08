#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color: vec4<f32>;
/// x=rotation_radians  y=falloff_k  z=inner_radius  w=unused
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> sprite_params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var gobo_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var gobo_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // UV is 0..1 across the quad. Centre = (0.5, 0.5).
    let uv = in.uv - vec2<f32>(0.5);
    let r = length(uv) * 2.0; // 0 at centre, 1 at edge

    // Hard edge fade near the disc perimeter.
    let edge = 1.0 - smoothstep(0.85, 1.0, r);
    if edge < 0.01 { return vec4<f32>(0.0); }

    // Radial falloff for soft glow.
    let falloff = exp(-r * r * sprite_params.y);

    // Gobo rotation + mask (optional detail even at small sizes).
    let rot = sprite_params.x;
    let c = cos(rot);
    let s = sin(rot);
    let gobo_uv = vec2<f32>(uv.x*c - uv.y*s, uv.x*s + uv.y*c) + vec2<f32>(0.5);
    let gobo_mask = textureSample(gobo_texture, gobo_sampler, gobo_uv).r;

    let alpha = falloff * edge * gobo_mask;
    return vec4<f32>(color.rgb, alpha * color.a);
}
