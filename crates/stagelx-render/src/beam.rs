// Phase 2: volumetric beam cone rendering.
//
// Plan:
//   BeamBundle — cone mesh + custom BeamMaterial (additive blending)
//   BeamMaterial — wgpu Material impl: color, intensity, beam_angle, gobo_texture
//   Update system: reads DmxBuffer values → updates BeamMaterial uniforms each frame
//   Fog density driven by per-scene atmosphere setting
