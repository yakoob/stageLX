//! OSC input via `rosc` over UDP.
//!
//! Message schema handled here:
//!   /fixture/{id}/{attr}   f32   — set attribute 0.0–1.0 on all patched fixtures
//!   /fixture/{id}/color    fff   — set RGB 0.0–1.0
//!
//! The socket runs non-blocking in Bevy's Update loop (same pattern as Art-Net TX).
//! A background thread is used for blocking recv so the main thread is never stalled.

use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender};
use rosc::{OscPacket, OscType};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use stagelx_state::{IoConfig, Programmer, ProtocolStatus};

// ─── Incoming message ──────────────────────────────────────────────────────────

struct OscMsg {
    addr: String,
    args: Vec<OscType>,
}

// ─── Resource ─────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct OscState {
    rx: Receiver<OscMsg>,
    tx: Sender<OscMsg>,
    pub bound_port: Option<u16>,
    /// Clone of the socket held so we can shut it down when disabled.
    socket: Option<Arc<UdpSocket>>,
    /// Signal for the background thread to exit.
    shutdown: Arc<AtomicBool>,
}

impl Default for OscState {
    fn default() -> Self {
        let (tx, rx) = bounded(256);
        Self { rx, tx, bound_port: None, socket: None, shutdown: Arc::new(AtomicBool::new(false)) }
    }
}

// ─── Systems ──────────────────────────────────────────────────────────────────

/// Open / close the UDP socket based on IoConfig.
pub fn osc_manage_socket(mut state: ResMut<OscState>, mut cfg: ResMut<IoConfig>) {
    let want_open = cfg.osc_enabled;

    if want_open && state.bound_port.is_none() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), cfg.osc_port);
        match UdpSocket::bind(addr) {
            Ok(sock) => {
                // Use a short read timeout so the thread can poll the shutdown flag.
                sock.set_read_timeout(Some(Duration::from_millis(100))).ok();
                let sock = Arc::new(sock);
                let tx = state.tx.clone();
                let thread_sock = Arc::clone(&sock);
                let shutdown = Arc::clone(&state.shutdown);
                std::thread::spawn(move || {
                    let mut buf = vec![0u8; 1536];
                    loop {
                        match thread_sock.recv(&mut buf) {
                            Ok(n) => {
                                if let Ok(pkt) = rosc::decoder::decode(&buf[..n]) {
                                    forward_packet(pkt, &tx);
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                                if shutdown.load(Ordering::Relaxed) {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });
                info!("OSC listening on {}", addr);
                cfg.osc_status = ProtocolStatus::Live;
                state.bound_port = Some(cfg.osc_port);
                state.socket = Some(sock);
            }
            Err(_e) => {
                cfg.osc_status = ProtocolStatus::Error;
            }
        }
    }

    if !want_open && state.bound_port.is_some() {
        // Signal the background thread to exit.
        state.shutdown.store(true, Ordering::Relaxed);
        state.socket = None;
        state.bound_port = None;
        state.shutdown = Arc::new(AtomicBool::new(false));
        cfg.osc_status = ProtocolStatus::Idle;
    }
}

fn forward_packet(pkt: OscPacket, tx: &Sender<OscMsg>) {
    match pkt {
        OscPacket::Message(m) => {
            let _ = tx.try_send(OscMsg { addr: m.addr, args: m.args });
        }
        OscPacket::Bundle(b) => {
            for p in b.content {
                forward_packet(p, tx);
            }
        }
    }
}

/// Drain received OSC messages and apply them to the Programmer.
pub fn osc_receive(
    state: Res<OscState>,
    mut programmer: ResMut<Programmer>,
    mut cfg: ResMut<IoConfig>,
) {
    let mut count = 0u64;
    while let Ok(msg) = state.rx.try_recv() {
        // Parse /fixture/{id}/{attr}
        let parts: Vec<&str> = msg.addr.trim_start_matches('/').split('/').collect();
        if parts.len() >= 3 && parts[0] == "fixture" {
            let attr = parts[2];
            match attr {
                "color" => {
                    // Expects three float args
                    let floats: Vec<f32> = msg.args.iter().filter_map(osc_float).collect();
                    if floats.len() >= 3 {
                        programmer.color = [
                            floats[0].clamp(0.0, 1.0),
                            floats[1].clamp(0.0, 1.0),
                            floats[2].clamp(0.0, 1.0),
                        ];
                    }
                }
                _ => {
                    if let Some(val) = msg.args.first().and_then(osc_float) {
                        let val = val.clamp(0.0, 1.0);
                        match attr {
                            "dimmer" => programmer.dimmer      = val,
                            "pan"    => programmer.pan         = val,
                            "tilt"   => programmer.tilt        = val,
                            "zoom"   => programmer.zoom        = val,
                            "strobe" => programmer.strobe      = val,
                            "red"    => programmer.color[0]    = val,
                            "green"  => programmer.color[1]    = val,
                            "blue"   => programmer.color[2]    = val,
                            _        => {}
                        }
                    }
                }
            }
            count += 1;
        }
    }
    cfg.osc_rx_count = cfg.osc_rx_count.saturating_add(count);
}

fn osc_float(t: &OscType) -> Option<f32> {
    match t {
        OscType::Float(f)  => Some(*f),
        OscType::Double(d) => Some(*d as f32),
        OscType::Int(i)    => Some(*i as f32),
        _                  => None,
    }
}
