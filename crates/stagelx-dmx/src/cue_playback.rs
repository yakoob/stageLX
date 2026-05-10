//! Cue playback → DMX channel projection.
//!
//! Writes the active cue's snapshot into the DMX engine at priority 150
//! (below programmer=200, above external input=100).

use bevy::prelude::*;
use stagelx_patch::PatchRes;
use stagelx_show::{CuePlayhead, CueStack};

use crate::engine::DmxEngineRes;
use crate::merge::MergeStrategy;

/// Write the active cue's snapshot into the DMX engine's "cue_playback" source.
/// Runs in FixedUpdate alongside programmer_to_dmx.
pub fn cue_to_dmx(
    mut engine: ResMut<DmxEngineRes>,
    stack: Res<CueStack>,
    playhead: Res<CuePlayhead>,
    patch: Res<PatchRes>,
) {
    let source = engine
        .0
        .get_or_add_source("cue_playback", 150, MergeStrategy::Ltp);

    let Some(idx) = playhead.current_cue_index else { return };
    let Some(cue) = stack.cues.get(idx) else { return };

    for (fixture_id, values) in &cue.snapshot {
        let Some(inst) = patch.0.get(*fixture_id) else { continue };
        let base = inst.address.channel;
        let universe = inst.address.universe;
        let buf = source.universes.get_or_insert(universe);

        let dimmer_byte = (values.dimmer.clamp(0.0, 1.0) * 255.0) as u8;
        let pan_raw = (values.pan.clamp(0.0, 1.0) * 65535.0) as u16;
        let tilt_raw = (values.tilt.clamp(0.0, 1.0) * 65535.0) as u16;
        let r = (values.color[0].clamp(0.0, 1.0) * 255.0) as u8;
        let g = (values.color[1].clamp(0.0, 1.0) * 255.0) as u8;
        let b = (values.color[2].clamp(0.0, 1.0) * 255.0) as u8;

        // Use pre-computed channel map when available; fall back to generic 8-ch layout.
        let has_map = inst.channel_map.dimmer.is_some()
            || inst.channel_map.pan.is_some()
            || inst.channel_map.tilt.is_some();

        if has_map {
            if let Some(off) = inst.channel_map.dimmer {
                buf.set(base + off, dimmer_byte);
            }
            if let Some(off) = inst.channel_map.pan {
                buf.set(base + off, (pan_raw >> 8) as u8);
            }
            if let Some(off) = inst.channel_map.pan_fine {
                buf.set(base + off, (pan_raw & 0xFF) as u8);
            }
            if let Some(off) = inst.channel_map.tilt {
                buf.set(base + off, (tilt_raw >> 8) as u8);
            }
            if let Some(off) = inst.channel_map.tilt_fine {
                buf.set(base + off, (tilt_raw & 0xFF) as u8);
            }
            if let Some(off) = inst.channel_map.red {
                buf.set(base + off, r);
            }
            if let Some(off) = inst.channel_map.green {
                buf.set(base + off, g);
            }
            if let Some(off) = inst.channel_map.blue {
                buf.set(base + off, b);
            }
        } else {
            // Generic 8-ch: Dimmer | Pan MSB | Pan Fine | Tilt MSB | Tilt Fine | R | G | B
            buf.set(base, dimmer_byte);
            buf.set(base + 1, (pan_raw >> 8) as u8);
            buf.set(base + 2, (pan_raw & 0xFF) as u8);
            buf.set(base + 3, (tilt_raw >> 8) as u8);
            buf.set(base + 4, (tilt_raw & 0xFF) as u8);
            buf.set(base + 5, r);
            buf.set(base + 6, g);
            buf.set(base + 7, b);
        }
    }
}
