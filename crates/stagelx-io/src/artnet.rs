use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::atomic::{AtomicU64, Ordering},
    sync::Arc,
};

use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use stagelx_dmx::engine::DmxEngineRes;
use stagelx_state::{IoConfig, ProtocolStatus};

use crate::supervisor::IoSupervisor;

pub const ARTNET_PORT: u16 = 6454;
// Cap the number of universes accepted from any single Art-Net source to
// prevent memory amplification from a malicious or misbehaving sender.
const MAX_RX_UNIVERSES: usize = 64;

// ─── ArtDMX packet helpers ────────────────────────────────────────────────────

fn build_artdmx(universe: u16, data: &[u8; 512]) -> Vec<u8> {
    let mut pkt = Vec::with_capacity(530);
    pkt.extend_from_slice(b"Art-Net\0");
    // OpCode: ArtDMX = 0x5000, little-endian
    pkt.push(0x00);
    pkt.push(0x50);
    // Protocol version 14, big-endian
    pkt.push(0x00);
    pkt.push(14);
    // Sequence (0 = disabled), Physical
    pkt.push(0);
    pkt.push(0);
    // SubUni / Net (Art-Net 3 encoding)
    pkt.push((universe & 0xFF) as u8);
    pkt.push(((universe >> 8) & 0x7F) as u8);
    // Length (512), big-endian
    pkt.push(0x02);
    pkt.push(0x00);
    pkt.extend_from_slice(data);
    pkt
}

/// Returns `(universe, dmx_data)` for a valid ArtDMX packet, otherwise `None`.
fn parse_artdmx(buf: &[u8]) -> Option<(u16, &[u8])> {
    if buf.len() < 18 {
        return None;
    }
    if &buf[..8] != b"Art-Net\0" {
        return None;
    }
    let opcode = u16::from_le_bytes([buf[8], buf[9]]);
    if opcode != 0x5000 {
        return None;
    }
    let universe = (buf[14] as u16) | ((buf[15] as u16 & 0x7F) << 8);
    let length = u16::from_be_bytes([buf[16], buf[17]]) as usize;
    if length == 0 || buf.len() < 18 + length {
        return None;
    }
    Some((universe, &buf[18..18 + length]))
}

// ─── Received packet ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ReceivedPacket {
    pub universe: u16,
    pub source: IpAddr,
    /// Fixed-size DMX data — avoids heap allocation on the UDP hot path.
    pub data: [u8; 512],
}

// ─── Bevy Resources ───────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct ArtNetState {
    pub socket: Option<UdpSocket>,
    pub rx_chan: Option<Receiver<ReceivedPacket>>,
    rx_thread_tx: Option<Sender<ReceivedPacket>>,
    /// Parsed cache of IoConfig::artnet_allowed_sources. Rebuilt only when
    /// the source string changes, avoiding a Vec allocation every frame.
    pub cached_allowlist: Vec<IpAddr>,
    cached_allowlist_src: String,
    /// Shared drop counter incremented by the RX thread when the channel is full.
    pub rx_drops: Arc<AtomicU64>,
    /// Throttle bind retries to avoid log-spam when the port is busy.
    last_bind_attempt: Option<std::time::Instant>,
}

impl Default for ArtNetState {
    fn default() -> Self {
        Self {
            socket: None,
            rx_chan: None,
            rx_thread_tx: None,
            cached_allowlist: Vec::new(),
            cached_allowlist_src: String::new(),
            rx_drops: Arc::new(AtomicU64::new(0)),
            last_bind_attempt: None,
        }
    }
}

// ─── Systems ──────────────────────────────────────────────────────────────────

/// Manage the UDP socket lifetime based on IoConfig.
pub fn artnet_manage_socket(
    mut state: ResMut<ArtNetState>,
    cfg: Res<IoConfig>,
    supervisor: Res<IoSupervisor>,
) {
    if state.socket.is_none() {
        let now = std::time::Instant::now();
        let should_try = state.last_bind_attempt.map_or(true, |t| {
            now.duration_since(t).as_secs_f32() >= 1.0
        });
        if should_try {
            state.last_bind_attempt = Some(now);
            let bind_ip: IpAddr = cfg
                .artnet_ip
                .trim()
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
            let bind_addr = SocketAddr::new(bind_ip, ARTNET_PORT);
            match UdpSocket::bind(bind_addr) {
                Ok(sock) => {
                    sock.set_nonblocking(true).ok();
                    sock.set_broadcast(true).ok();
                    info!("Art-Net socket bound to {}", bind_addr);
                    state.socket = Some(sock);
                }
                Err(e) => warn!("Art-Net bind failed: {e}"),
            }
        }
    }

    if cfg.artnet_rx_enabled && state.rx_chan.is_none() {
        // Clone the bound TX socket so both sides share the same port binding.
        // A second bind() to the same port would fail with EADDRINUSE.
        if let Some(ref sock) = state.socket {
            match sock.try_clone() {
                Ok(rx_sock) => {
                    // TX socket is nonblocking (send_to must not stall the frame).
                    // The clone inherits that mode — reset it so the RX thread
                    // blocks in the kernel until a packet arrives instead of
                    // busy-polling with sleep(1ms).
                    rx_sock.set_nonblocking(false).ok();

                    let (tx, rx) = bounded::<ReceivedPacket>(8);
                    state.rx_chan = Some(rx);
                    state.rx_thread_tx = Some(tx.clone());
                    let drops = state.rx_drops.clone();

                    std::thread::spawn(move || {
                        let mut buf = [0u8; 600];
                        loop {
                            match rx_sock.recv_from(&mut buf) {
                                Ok((n, src)) => {
                                    if let Some((universe, data)) = parse_artdmx(&buf[..n]) {
                                        let mut pkt_data = [0u8; 512];
                                        let len = data.len().min(512);
                                        pkt_data[..len].copy_from_slice(&data[..len]);
                                        let pkt = ReceivedPacket {
                                            universe,
                                            source: src.ip(),
                                            data: pkt_data,
                                        };
                                        if let Err(TrySendError::Full(_)) = tx.try_send(pkt) {
                                            drops.fetch_add(1, Ordering::Relaxed);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Art-Net RX error: {e}");
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                }
                            }
                        }
                    });
                }
                Err(e) => warn!("Art-Net RX socket clone failed: {e}"),
            }
        }
    }

    if !cfg.artnet_rx_enabled {
        state.rx_chan = None;
        state.rx_thread_tx = None;
    }

    // Rebuild the allowlist cache only when the config string actually changed.
    if cfg.artnet_allowed_sources != state.cached_allowlist_src {
        state.cached_allowlist = cfg
            .artnet_allowed_sources
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        state.cached_allowlist_src.clone_from(&cfg.artnet_allowed_sources);
    }

    // Sync thread-local drops into the supervisor.
    let local_drops = state.rx_drops.load(Ordering::Relaxed);
    let global = supervisor.rx_drops.load(Ordering::Relaxed);
    if local_drops > global {
        supervisor.rx_drops.store(local_drops, Ordering::Relaxed);
    }
}

/// Drain incoming Art-Net packets into the DMX engine "artnet_in" source.
///
/// Security mitigations applied here:
/// - Source IP allowlist: packets from unlisted hosts are silently dropped.
/// - Universe cap: at most MAX_RX_UNIVERSES unique universes are created per source
///   to prevent the memory-amplification attack from a malicious sender.
pub fn artnet_receive(
    state: Res<ArtNetState>,
    mut engine: ResMut<DmxEngineRes>,
    mut cfg: ResMut<IoConfig>,
) {
    let Some(rx) = &state.rx_chan else { return };

    // Use the pre-parsed allowlist from ArtNetState (rebuilt only on config change).
    let allowlist = &state.cached_allowlist;

    let source = engine
        .0
        .get_or_add_source("artnet_in", 100, stagelx_dmx::merge::MergeStrategy::Htp);

    let current_universe_count = source.universes.universes().count();
    let mut new_universe_count = current_universe_count;
    let mut count = 0u64;

    while let Ok(pkt) = rx.try_recv() {
        if !allowlist.is_empty() && !allowlist.contains(&pkt.source) {
            continue;
        }

        // #2 — Universe cap
        let already_exists = source.universes.get(pkt.universe).is_some();
        if !already_exists && new_universe_count >= MAX_RX_UNIVERSES {
            continue;
        }
        if !already_exists {
            new_universe_count += 1;
        }

        let buf = source.universes.get_or_insert(pkt.universe);
        buf.copy_from_slice(&pkt.data);
        count += 1;
    }
    cfg.artnet_rx_count = cfg.artnet_rx_count.saturating_add(count);
}

/// Merge all DMX sources into the output universe set.
/// Must run after programmer_to_dmx and artnet_receive, before the send systems.
pub fn dmx_engine_tick(mut engine: ResMut<DmxEngineRes>) {
    engine.0.tick();
}

/// Send Art-Net output for the configured universe.
/// Runs in FixedUpdate — no Instant throttle needed.
pub fn artnet_send(
    state: Res<ArtNetState>,
    engine: Res<DmxEngineRes>,
    mut cfg: ResMut<IoConfig>,
) {

    let Some(sock) = &state.socket else { return };

    // #4 — Configurable TX destination (empty = limited broadcast)
    let dest_ip: IpAddr = cfg
        .artnet_dest_ip
        .trim()
        .parse()
        .unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST));
    let dest = SocketAddr::new(dest_ip, ARTNET_PORT);

    let out_universe = cfg.artnet_out_universe;
    if let Some(dmx_buf) = engine.0.output_buffer(out_universe) {
        let pkt = build_artdmx(out_universe, dmx_buf.as_bytes());
        match sock.send_to(&pkt, dest) {
            Ok(_) => {
                cfg.artnet_tx_count = cfg.artnet_tx_count.saturating_add(1);
                cfg.artnet_status = ProtocolStatus::Live;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_e) => {
                cfg.artnet_status = ProtocolStatus::Error;
            }
        }
    }
}
