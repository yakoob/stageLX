//! Unified show file format (.slx).
//!
//! Bundles patch, cue stack, venue path, and metadata into a single JSON file.
//! Versioned for future format migrations.

use serde::{Deserialize, Serialize};
use stagelx_core::patch::Patch;
use crate::CueStack;

/// Current show file format version.
const VERSION: u32 = 1;
const PATH: &str = "show.slx";

/// Legacy cue-only file path.
const LEGACY_PATH: &str = "show.json";

/// A complete show file — everything needed to restore a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowFile {
    pub version: u32,
    pub patch: Patch,
    pub cue_stack: CueStack,
    pub venue_path: Option<String>,
    pub name: String,
}

impl Default for ShowFile {
    fn default() -> Self {
        Self {
            version: VERSION,
            patch: Patch::default(),
            cue_stack: CueStack::default(),
            venue_path: None,
            name: "Untitled Show".into(),
        }
    }
}

impl ShowFile {
    /// Save to the default show file path.
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(PATH, json)
    }

    /// Load from the default show file path.
    /// Falls back to legacy `show.json` (cue-only) if `.slx` doesn't exist.
    pub fn load() -> Option<Self> {
        // Try modern format first.
        if let Ok(bytes) = std::fs::read(PATH) {
            return serde_json::from_slice(&bytes).ok();
        }

        // Backward-compat: migrate legacy cue-only JSON.
        if let Ok(bytes) = std::fs::read(LEGACY_PATH) {
            if let Ok(cue_stack) = serde_json::from_slice::<CueStack>(&bytes) {
                return Some(Self {
                    version: VERSION,
                    patch: Patch::default(),
                    cue_stack,
                    venue_path: None,
                    name: "Migrated Show".into(),
                });
            }
        }

        None
    }

    /// Save to an explicit path (used by File → Save As).
    pub fn save_to(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    /// Load from an explicit path (used by File → Open).
    pub fn load_from(path: &str) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }
}

/// Events for show file I/O.
use bevy::prelude::*;
use crate::VenueLoadState;

/// Triggered by UI (Ctrl+S or File → Save).
#[derive(Event, Debug, Clone)]
pub struct SaveShowEvent;

/// Triggered by UI (Ctrl+O or File → Open).
#[derive(Event, Debug, Clone)]
pub struct LoadShowEvent {
    pub path: String,
}

/// Observer: saves the current show state to `show.slx`.
pub fn on_save_show(
    _trigger: On<SaveShowEvent>,
    patch: Res<stagelx_patch::PatchRes>,
    stack: Res<CueStack>,
    venue: Res<VenueLoadState>,
) {
    let show = ShowFile {
        version: VERSION,
        patch: patch.0.clone(),
        cue_stack: stack.clone(),
        venue_path: if venue.import_path.is_empty() {
            None
        } else {
            Some(venue.import_path.clone())
        },
        name: "show".into(),
    };
    if let Err(e) = show.save() {
        warn!("Failed to save show: {}", e);
    } else {
        info!("Show saved to {}", PATH);
    }
}

/// Observer: loads a show from an explicit path (File → Open).
pub fn on_load_show(
    trigger: On<LoadShowEvent>,
    mut patch: ResMut<stagelx_patch::PatchRes>,
    mut stack: ResMut<CueStack>,
    mut venue: ResMut<VenueLoadState>,
) {
    let path = &trigger.event().path;
    if let Some(show) = ShowFile::load_from(path) {
        patch.0 = show.patch;
        *stack = show.cue_stack;
        if let Some(vp) = show.venue_path {
            venue.import_path = vp;
        }
        info!("Show loaded from {} (v{})", path, show.version);
    } else {
        warn!("Failed to load show from {}", path);
    }
}

/// Startup system: auto-load `show.slx` (or migrate legacy `show.json`) on app start.
pub fn auto_load_show_on_startup(
    mut patch: ResMut<stagelx_patch::PatchRes>,
    mut stack: ResMut<CueStack>,
    mut venue: ResMut<VenueLoadState>,
) {
    if let Some(show) = ShowFile::load() {
        patch.0 = show.patch;
        *stack = show.cue_stack;
        if let Some(path) = show.venue_path {
            venue.import_path = path;
        }
        info!("Show loaded from {} (v{})", PATH, show.version);
    }
}
