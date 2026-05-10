//! Per-protocol telemetry Resources.
//!
//! These are written by the IO systems (often per-frame) and read by the UI.
//! Splitting them from Config eliminates exclusive `ResMut` contention between
//! the IO send/receive systems and the config UI panel.

use bevy::prelude::*;
use stagelx_show::ProtocolStatus;

// ─── Art-Net ──────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct ArtNetStats {
    pub tx_count: u64,
    pub rx_count: u64,
    pub status: ProtocolStatus,
}

// ─── sACN (E1.31) ─────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct SacnStats {
    pub tx_count: u64,
    pub rx_count: u64,
    pub status: ProtocolStatus,
}

// ─── USB DMX (Enttec USB Pro) ─────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct UsbStats {
    pub tx_count: u64,
    pub status: ProtocolStatus,
}

// ─── MIDI input ───────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct MidiStats {
    pub rx_count: u64,
    pub status: ProtocolStatus,
}

// ─── OSC input ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct OscStats {
    pub rx_count: u64,
    pub status: ProtocolStatus,
}
