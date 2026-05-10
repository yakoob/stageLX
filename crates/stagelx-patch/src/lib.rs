//! Patch-level Bevy Resources and Events for stageLX.
//!
//! Contains fixture patch data, DMX address types, and lifecycle events
//! emitted when fixtures are added or removed from the show.

use bevy::prelude::*;
use stagelx_core::{patch::Patch, types::FixtureId};

// Re-exports from core (convenience so downstream crates don't depend on both)
pub use stagelx_core::fixture::FixtureInstance;
pub use stagelx_core::types::DmxAddress;

// ─── Events ───────────────────────────────────────────────────────────────────

/// Emitted by the patch UI or MVR importer when a fixture is added to the patch.
/// The render plugin responds by spawning the 3D scene entity.
#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnFixtureEvent(pub FixtureId);

/// Emitted when a fixture is removed from the patch.
/// The render plugin responds by despawning the corresponding scene entity.
#[derive(Event, Debug, Clone, Copy)]
pub struct DespawnFixtureEvent(pub FixtureId);

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
