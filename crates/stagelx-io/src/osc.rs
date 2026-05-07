// Phase 5: OSC input via rosc (UDP, default port 8000).
//
// Planned message schema:
//   /fixture/{id}/{attribute}  f32   → set attribute value (0.0–1.0)
//   /fixture/{id}/color        fff   → set RGB (0.0–1.0 each)
//   /cue/{id}/go               →     → trigger cue (future)
