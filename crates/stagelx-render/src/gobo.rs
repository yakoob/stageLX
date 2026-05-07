// Phase 2: gobo projection.
//
// Plan:
//   GoboProjector — component holding current wheel slot texture handle
//   load_gobo_texture() — loads PNG from GDTF ZIP asset into Bevy asset server
//   update_gobo() — system: on GoboIndex channel change, swaps active texture
//   Projection via custom spotlight shader pass (projected texture into beam cone)
//   GoboRotation attribute drives UV rotation in the shader
