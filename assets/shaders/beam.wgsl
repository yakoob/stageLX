#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> gobo_params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var gobo_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var gobo_sampler: sampler;
/// x=half_angle_rad  y=cone_length_m  z=scatter_k  w=extinction_k
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> beam_params: vec4<f32>;
/// world→cone-local inverse transform, written each frame by articulate_beams
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> world_to_cone: mat4x4<f32>;
/// Ray-march step count. 16 = Tier 1, 32 = Tier 2.
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var<uniform> step_count: i32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Perspective cameras (FOH) cast rays from a single eye; orthographic
    // cameras (TOP/SIDE previews) cast parallel rays. The standard test for an
    // orthographic projection is clip_from_view[3][3] == 1.0.
    let is_ortho = view.clip_from_view[3][3] == 1.0;

    // Ray direction into the scene — used for the back-face test and the march.
    var view_dir: vec3<f32>;
    if is_ortho {
        // Camera forward = -Z column of the camera's world transform.
        view_dir = normalize(-view.world_from_view[2].xyz);
    } else {
        view_dir = normalize(in.world_position.xyz - view.world_position);
    }

    // With cull_mode=None both faces rasterize.  Process only the back face
    // (where the ray exits the cone) so each screen pixel marches once.
    if dot(view_dir, normalize(in.world_normal)) < 0.0 {
        return vec4<f32>(0.0);
    }

    // Gobo UV rotation around (0.5, 0.5).
    var uv = in.uv - vec2<f32>(0.5);
    let c = cos(gobo_params.x);
    let s = sin(gobo_params.x);
    uv = vec2<f32>(uv.x*c - uv.y*s, uv.x*s + uv.y*c) + vec2<f32>(0.5);
    let gobo_mask = textureSample(gobo_texture, gobo_sampler, uv).r;
    if gobo_mask < 0.01 { return vec4<f32>(0.0); }

    let ta = tan(beam_params.x);   // tan(half_angle)
    let H  = beam_params.y;         // cone length (metres)
    let sk = beam_params.z;         // radial scatter falloff
    let ek = beam_params.w;         // depth extinction falloff

    // Build the view ray as two world-space points A→B, then map both into
    // cone-local space (so the cone's non-uniform XZ zoom scale is handled).
    // B = this fragment (the cone's exit face). For perspective A is the eye;
    // orthographic has no eye, so A is a point pushed back along the parallel
    // ray, far enough to sit in front of the cone (stage is tens of metres).
    var a_world: vec3<f32>;
    if is_ortho {
        a_world = in.world_position.xyz - view_dir * 100.0;
    } else {
        a_world = view.world_position;
    }

    // Transform ray into cone-local space.  t=0 = A, t=1 = this fragment.
    let o  = (world_to_cone * vec4<f32>(a_world, 1.0)).xyz;
    let fp = (world_to_cone * vec4<f32>(in.world_position.xyz, 1.0)).xyz;
    let d  = fp - o;

    // Ray–cone quadratic.
    // Cone: apex=(0, H/2, 0), axis=(0,−1,0), surface: x²+z² = ((H/2−y)·ta)²
    let K   = H * 0.5 - o.y;
    let A   = d.x*d.x + d.z*d.z - ta*ta*d.y*d.y;
    let B   = 2.0*(o.x*d.x + o.z*d.z + ta*ta*K*d.y);
    let Cv  = o.x*o.x + o.z*o.z - ta*ta*K*K;
    let dsc = B*B - 4.0*A*Cv;
    if dsc < 0.0 || abs(A) < 1e-8 { return vec4<f32>(0.0); }

    let sq    = sqrt(dsc);
    let inv2A = 0.5 / A;
    var t0 = (-B - sq)*inv2A;
    var t1 = (-B + sq)*inv2A;
    if t0 > t1 { let tmp = t0; t0 = t1; t1 = tmp; }

    // Clamp to segment camera→fragment and to finite cone Y band.
    t0 = max(t0, 0.0);
    t1 = min(t1, 1.0);
    if abs(d.y) > 1e-6 {
        let tlo = (-H*0.5 - o.y) / d.y;
        let thi = ( H*0.5 - o.y) / d.y;
        t0 = max(t0, min(tlo, thi));
        t1 = min(t1, max(tlo, thi));
    }
    if t1 <= t0 + 1e-6 { return vec4<f32>(0.0); }

    // Dynamic-step ray march accumulating volumetric density.
    var acc   = 0.0;
    let n     = step_count;
    let inv_n = 1.0 / f32(n);
    let chord = t1 - t0;
    for (var i = 0; i < n; i = i + 1) {
        let t   = t0 + (f32(i) + 0.5) * chord * inv_n;
        let p   = o + t * d;
        let r_c = max((H*0.5 - p.y) * ta, 1e-6);
        let r   = sqrt(p.x*p.x + p.z*p.z);
        if r > r_c { continue; }
        let nr  = r / r_c;                              // 0 at axis, 1 at surface
        let dn  = (t - t0) / max(chord, 1e-6);         // 0 at entry, 1 at exit
        acc += exp(-nr*nr * sk) * exp(-dn * ek);
    }

    // Average sample density, then scale for perceptible brightness.
    acc = clamp(acc * inv_n * 1.5, 0.0, 1.0);

    // The transparent phase uses PREMULTIPLIED-alpha blending
    // (result = src.rgb + dst.rgb·(1−src.a)), so RGB must be premultiplied by
    // the accumulated density.  Emitting full-bright RGB with the density only
    // in the alpha channel makes every covered pixel add full colour regardless
    // of density — producing solid, blown-out white cones.
    let a = acc * color.a;
    return vec4<f32>(color.rgb * gobo_mask * a, a);
}
