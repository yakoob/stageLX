//! MIDI input via `midir`.
//!
//! CC messages are mapped to normalised programmer attributes (0.0–1.0).
//! The port connection is held as a NonSend resource because MidiInputConnection
//! is Send but not Sync.

use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender};
use midir::MidiInput;
use stagelx_state::{Programmer, ProtocolStatus};
use crate::config::MidiConfig;
use crate::stats::MidiStats;

// ─── State (NonSend — MidiInputConnection is !Sync) ───────────────────────────

pub struct MidiState {
    pub connection: Option<midir::MidiInputConnection<()>>,
    pub rx: Receiver<[u8; 3]>,
    tx: Sender<[u8; 3]>,
    /// Cached list of available port names, refreshed at most 1 Hz.
    pub port_names: Vec<String>,
    /// Last time port names were scanned (seconds).
    last_scan_time: f32,
}

impl Default for MidiState {
    fn default() -> Self {
        let (tx, rx) = bounded(256);
        Self { connection: None, rx, tx, port_names: Vec::new(), last_scan_time: 0.0 }
    }
}

// ─── Systems ──────────────────────────────────────────────────────────────────

/// Refresh the port list and open/close the connection based on IoConfig.
/// Port scanning is rate-limited to ≤ 1 Hz (Rule 15).
pub fn midi_manage_connection(
    mut state: NonSendMut<MidiState>,
    cfg: ResMut<MidiConfig>,
    mut stats: ResMut<MidiStats>,
    time: Res<Time>,
) {
    // Rate-limit port scan to 1 Hz.
    let now = time.elapsed_secs();
    if now - state.last_scan_time >= 1.0 {
        state.last_scan_time = now;
        // Refresh port names (requires a throw-away MidiInput — the API is consuming).
        if let Ok(mi) = MidiInput::new("stageLX-scan") {
            let ports = mi.ports();
            state.port_names = ports.iter()
                .filter_map(|p| mi.port_name(p).ok())
                .collect();
        }
    }

    let want_open = cfg.enabled && !cfg.port.trim().is_empty();

    if want_open && state.connection.is_none() {
        let port_name = cfg.port.trim().to_string();
        let tx = state.tx.clone();
        match open_midi_port(&port_name, tx) {
            Ok(conn) => {
                info!("MIDI connected: {}", port_name);
                stats.status = ProtocolStatus::Live;
                state.connection = Some(conn);
            }
            Err(_e) => {
                stats.status = ProtocolStatus::Error;
            }
        }
    }

    if !want_open && state.connection.is_some() {
        state.connection = None;
        stats.status = ProtocolStatus::Idle;
        info!("MIDI disconnected");
    }
}

/// Drain received CC messages and apply them to the Programmer resource.
pub fn midi_receive(
    state: NonSendMut<MidiState>,
    mut programmer: ResMut<Programmer>,
    cfg: Res<MidiConfig>,
    mut stats: ResMut<MidiStats>,
) {
    let mut count = 0u64;
    while let Ok(msg) = state.rx.try_recv() {
        let status = msg[0] & 0xF0;
        if status != 0xB0 { continue; } // CC only
        let cc  = msg[1];
        let val = msg[2] as f32 / 127.0;

        if      cc == cfg.cc_dimmer { programmer.dimmer     = val; }
        else if cc == cfg.cc_pan    { programmer.pan        = val; }
        else if cc == cfg.cc_tilt   { programmer.tilt       = val; }
        else if cc == cfg.cc_red    { programmer.color[0]   = val; }
        else if cc == cfg.cc_green  { programmer.color[1]   = val; }
        else if cc == cfg.cc_blue   { programmer.color[2]   = val; }
        else if cc == cfg.cc_zoom   { programmer.zoom       = val; }
        else if cc == cfg.cc_strobe { programmer.strobe     = val; }
        count += 1;
    }
    stats.rx_count = stats.rx_count.saturating_add(count);
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn open_midi_port(
    port_name: &str,
    tx: Sender<[u8; 3]>,
) -> Result<midir::MidiInputConnection<()>, Box<dyn std::error::Error>> {
    let mi = MidiInput::new("stageLX")?;
    let ports = mi.ports();
    let port = ports.iter()
        .find(|p| mi.port_name(p).ok().as_deref() == Some(port_name))
        .ok_or("MIDI port not found")?;
    let conn = mi.connect(port, "stageLX-rx", move |_, msg, _| {
        if msg.len() >= 3 {
            let _ = tx.try_send([msg[0], msg[1], msg[2]]);
        }
    }, ())?;
    Ok(conn)
}
