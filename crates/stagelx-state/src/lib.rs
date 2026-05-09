//! Shared Bevy Resources and Events for stageLX.
//!
//! Lives in its own crate so that stagelx-render, stagelx-io, and stagelx-ui
//! can all depend on the resource *types* without any of them depending on
//! each other.
//!
//! # FROZEN — do not add new Resources here
//!
//! New state belongs in its owning crate:
//! - MIDI / OSC config → `stagelx-io`
//! - Viewport layout → `stagelx-render`
//! - Export staging → `stagelx-export` (future)
//!
//! Phase 6 will extract `Programmer` → `stagelx-show` and `PatchRes` → `stagelx-patch`.

use bevy::prelude::*;
use stagelx_core::{patch::Patch, types::FixtureId};
use stagelx_gdtf::FixtureLibrary;

// ─── Events ───────────────────────────────────────────────────────────────────

/// Emitted by the patch UI or MVR importer when a fixture is added to the patch.
/// The render plugin responds by spawning the 3D scene entity.
#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnFixtureEvent(pub FixtureId);

/// Emitted when a fixture is removed from the patch.
/// The render plugin responds by despawning the corresponding scene entity.
#[derive(Event, Debug, Clone, Copy)]
pub struct DespawnFixtureEvent(pub FixtureId);

/// Emitted by the Library UI when the user loads a venue file.
/// The render plugin observes this and calls the actual mesh loader,
/// keeping `stagelx-ui` free of any `stagelx-render` dependency.
#[derive(Event, Debug, Clone)]
pub struct LoadVenueEvent {
    pub path: String,
}

// ─── Programmer ───────────────────────────────────────────────────────────────

/// Normalised programmer state — all channel values 0.0–1.0.
/// Written by the UI and keyboard handler; read by render and DMX output.
#[derive(Resource, Clone, PartialEq)]
pub struct Programmer {
    pub pan: f32,
    pub tilt: f32,
    pub dimmer: f32,
    pub color: [f32; 3],
    pub pan_range: f32,
    pub tilt_range: f32,
    /// 0.0 = narrowest beam (5°), 1.0 = widest beam (45°).
    pub zoom: f32,
    /// 0.0 = shutter open (no strobe), 1.0 = fastest strobe (~25 Hz).
    pub strobe: f32,
    /// Index into GoboLibrary (0 = open beam).
    pub gobo_index: usize,
    /// Gobo spin speed in rotations per second (0.0 = static).
    pub gobo_spin: f32,
}

impl Default for Programmer {
    fn default() -> Self {
        Self {
            pan: 0.5,
            tilt: 0.5,
            dimmer: 1.0,
            color: [1.0, 1.0, 1.0],
            pan_range: 540.0,
            tilt_range: 270.0,
            zoom: 0.0,
            strobe: 0.0,
            gobo_index: 0,
            gobo_spin: 0.0,
        }
    }
}

// ─── PatchRes ─────────────────────────────────────────────────────────────────

/// Bevy Resource wrapping the show patch (fixture → DMX address mapping).
#[derive(Resource, Default)]
pub struct PatchRes(pub Patch);

// ─── PatchEditState ───────────────────────────────────────────────────────────

/// Transient state for the patch panel "Add Fixture" form.
#[derive(Resource, Default)]
pub struct PatchEditState {
    pub selected_type_id: String,
    pub selected_mode:    String,
    pub new_name:         String,
    pub universe_str:     String,
    pub channel_str:      String,
    pub add_error:        Option<String>,
}

// ─── FixtureLibraryRes ────────────────────────────────────────────────────────

/// Bevy Resource wrapping the loaded GDTF fixture library.
#[derive(Resource, Default)]
pub struct FixtureLibraryRes {
    pub library: FixtureLibrary,
    /// Text field state for the GDTF import path input.
    pub import_path: String,
    pub import_error: Option<String>,
    /// MVR import state.
    pub mvr_import_path: String,
    pub mvr_import_error: Option<String>,
}

// ─── VenueLoadState ───────────────────────────────────────────────────────────

/// UI state for the venue loader (moved from stagelx-render per Rule 21).
#[derive(Resource, Default)]
pub struct VenueLoadState {
    pub import_path: String,
    pub import_error: Option<String>,
}

// ─── ProtocolStatus ───────────────────────────────────────────────────────────

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum ProtocolStatus {
    #[default]
    Idle,
    Live,
    Warn,
    Error,
}
