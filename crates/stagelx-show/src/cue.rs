//! Cue system data model and playback state.
//!
//! Phase 6.4 scope: basic cue stack, instant playback (no fade engine),
//! record-from-programmer, GO/BACK navigation, JSON persistence.

use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use stagelx_core::types::FixtureId;
use crate::Programmer;
use stagelx_patch::PatchRes;

// ─── Value snapshot ───────────────────────────────────────────────────────────

/// Normalised attribute values for a single fixture in a cue.
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CueValues {
    pub dimmer: f32,
    pub pan: f32,
    pub tilt: f32,
    pub zoom: f32,
    pub strobe: f32,
    pub color: [f32; 3],
}

impl CueValues {
    /// Capture current programmer values (programmer is global, so all fixtures
    /// get the same values in this Phase-6 snapshot).
    pub fn from_programmer(prog: &Programmer) -> Self {
        Self {
            dimmer: prog.dimmer,
            pan: prog.pan,
            tilt: prog.tilt,
            zoom: prog.zoom,
            strobe: prog.strobe,
            color: prog.color,
        }
    }
}

// ─── Cue ──────────────────────────────────────────────────────────────────────

/// A single cue in the stack.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cue {
    pub id: String,
    pub label: String,
    pub fade_in_ms: u32,
    pub fade_out_ms: u32,
    pub delay_ms: u32,
    /// fixture_id → attribute values.
    pub snapshot: HashMap<FixtureId, CueValues>,
}

impl Cue {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            fade_in_ms: 0,
            fade_out_ms: 0,
            delay_ms: 0,
            snapshot: HashMap::new(),
        }
    }
}

impl Default for Cue {
    fn default() -> Self {
        Self::new("1", "Untitled")
    }
}

// ─── CueStack ─────────────────────────────────────────────────────────────────

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct CueStack {
    pub cues: Vec<Cue>,
}

impl CueStack {
    /// Append a new cue captured from the current programmer state.
    /// Every fixture currently in the patch gets the same programmer values.
    pub fn record_from_programmer(
        &mut self,
        prog: &Programmer,
        patch: &PatchRes,
    ) -> usize {
        let values = CueValues::from_programmer(prog);
        let mut snapshot = HashMap::new();
        for inst in patch.0.fixtures() {
            snapshot.insert(inst.id, values.clone());
        }

        let num = self.cues.len() + 1;
        let cue = Cue {
            id: num.to_string(),
            label: format!("Cue {num}"),
            fade_in_ms: 0,
            fade_out_ms: 0,
            delay_ms: 0,
            snapshot,
        };
        self.cues.push(cue);
        self.cues.len() - 1
    }

    /// Delete a cue by index.
    pub fn delete_cue(&mut self, index: usize) {
        if index < self.cues.len() {
            self.cues.remove(index);
            // Renumber remaining cues.
            for (i, cue) in self.cues.iter_mut().enumerate() {
                cue.id = (i + 1).to_string();
            }
        }
    }

    /// Load from JSON file path. Returns Ok(()) even if file missing.
    pub fn load_from_file(path: &str) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Save to JSON file path.
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}

// ─── Playhead ─────────────────────────────────────────────────────────────────

#[derive(Resource, Default, Clone, Debug)]
pub struct CuePlayhead {
    pub current_cue_index: Option<usize>,
}

impl CuePlayhead {
    /// Advance to next cue. Returns the new index.
    pub fn go(&mut self, stack: &CueStack) -> Option<usize> {
        let next = match self.current_cue_index {
            Some(i) => (i + 1).min(stack.cues.len().saturating_sub(1)),
            None => 0,
        };
        if next < stack.cues.len() {
            self.current_cue_index = Some(next);
        }
        self.current_cue_index
    }

    /// Retreat to previous cue. Returns the new index.
    pub fn back(&mut self) -> Option<usize> {
        self.current_cue_index = match self.current_cue_index {
            Some(0) => None,
            Some(i) => Some(i - 1),
            None => None,
        };
        self.current_cue_index
    }
}

// ─── Events ───────────────────────────────────────────────────────────────────

/// Triggered by UI when user presses RECORD.
#[derive(Event, Debug, Clone)]
pub struct RecordCueEvent;

/// Triggered by UI when user presses GO.
#[derive(Event, Debug, Clone)]
pub struct GoCueEvent;

/// Triggered by UI when user presses BACK.
#[derive(Event, Debug, Clone)]
pub struct BackCueEvent;

/// Triggered by UI when user deletes a cue.
#[derive(Event, Debug, Clone, Copy)]
pub struct DeleteCueEvent(pub usize);

// ─── Observer handlers ────────────────────────────────────────────────────────

pub fn on_record_cue(
    _trigger: On<RecordCueEvent>,
    mut stack: ResMut<CueStack>,
    programmer: Res<Programmer>,
    patch: Res<PatchRes>,
) {
    stack.record_from_programmer(&programmer, &patch);
    let _ = stack.save_to_file("show.json");
}

pub fn on_go_cue(
    _trigger: On<GoCueEvent>,
    stack: Res<CueStack>,
    mut playhead: ResMut<CuePlayhead>,
) {
    playhead.go(&stack);
}

pub fn on_back_cue(
    _trigger: On<BackCueEvent>,
    mut playhead: ResMut<CuePlayhead>,
) {
    playhead.back();
}

pub fn on_delete_cue(
    trigger: On<DeleteCueEvent>,
    mut stack: ResMut<CueStack>,
    mut playhead: ResMut<CuePlayhead>,
) {
    let index = trigger.event().0;
    stack.delete_cue(index);
    // Adjust playhead if it pointed at or past the deleted cue.
    if let Some(idx) = playhead.current_cue_index {
        if idx >= stack.cues.len() {
            playhead.current_cue_index = stack.cues.len().checked_sub(1);
        } else if idx == index {
            playhead.current_cue_index = idx.checked_sub(1);
        }
    }
    let _ = stack.save_to_file("show.json");
}

/// Try to load show.json on startup.
pub fn load_cue_stack_on_startup(mut stack: ResMut<CueStack>) {
    if let Some(loaded) = CueStack::load_from_file("show.json") {
        *stack = loaded;
    }
}
