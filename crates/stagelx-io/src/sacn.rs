//! sACN / E1.31 (ANSI E1.31-2016) TX and RX.
//!
//! TX: sends Data Packets to the configured universe.
//!     Destination is the universe multicast address (239.255.hi.lo) unless
//!     `sacn_dest_ip` is set, in which case that unicast IP is used.
//!
//! RX: listens on UDP 5568. Multicast group joining (IP_ADD_MEMBERSHIP)
//!     is omitted for now — packets reach the socket on most managed-switch
//!     LANs via IGMP snooping or broadcast fallback.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    sync::Arc,
    time::Duration,
};

use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use stagelx_dmx::engine::DmxEngineRes;
use stagelx_state::ProtocolStatus;
use crate::config::SacnConfig;
use crate::stats::SacnStats;

use crate::supervisor::IoSupervisor;

pub const SACN_PORT: u16 = 5568;
const MAX_RX_UNIVERSES: usize = 64;

// Fixed Component Identifier for this source (ASCII "stageLX" padded).
const CID: [u8; 16] = *b"stageLX\x00\x00\x00\x00\x00\x00\x00\x00\x01";
// ACN Packet Identifier per ANSI E1.17.
const ACN_ID: &[u8; 12] = b"ASC-E1.17\x00\x00\x00";

// ─── E1.31 packet helpers ─────────────────────────────────────────────────────

/// Encode PDU Flags & Length: top 4 bits = 0x7, bottom 12 bits = (total - pdu_start).
fn fl(pdu_start: usize, total: usize) -> [u8; 2] {
    let len = total - pdu_start;
    [0x70 | ((len >> 8) as u8 & 0x0F), (len & 0xFF) as u8]
}

/// Build a 638-byte E1.31 Data Packet.
pub fn build_sacn(universe: u16, priority: u8, sequence: u8, data: &[u8; 512]) -> Vec<u8> {
    const TOTAL: usize = 638;
    let mut p = vec![0u8; TOTAL];

    // Preamble / post-amble / ACN identifier (bytes 0–15)
    p[0] = 0x00; p[1] = 0x10;          // Preamble Size
    p[2] = 0x00; p[3] = 0x00;          // Post-amble Size
    p[4..16].copy_from_slice(ACN_ID);  // ACN Packet Identifier

    // Root PDU (starts at 0x10 = 16)
    let [h, l] = fl(0x10, TOTAL);
    p[0x10] = h; p[0x11] = l;
    p[0x12..0x16].copy_from_slice(&[0x00, 0x00, 0x00, 0x04]); // VECTOR_ROOT_E131_DATA
    p[0x16..0x26].copy_from_slice(&CID);

    // Framing PDU (starts at 0x26 = 38)
    let [h, l] = fl(0x26, TOTAL);
    p[0x26] = h; p[0x27] = l;
    p[0x28..0x2C].copy_from_slice(&[0x00, 0x00, 0x00, 0x02]); // VECTOR_E131_DATA_PACKET
    let name = b"stageLX";
    p[0x2C..0x2C + name.len()].copy_from_slice(name); // Source Name (64 bytes, rest zero)
    p[0x6C] = priority;
    // Sync address = 0, Sequence number, Options = 0
    p[0x6F] = sequence;
    // Universe (big-endian)
    p[0x71] = (universe >> 8) as u8;
    p[0x72] = (universe & 0xFF) as u8;

    // DMP PDU (starts at 0x73 = 115)
    let [h, l] = fl(0x73, TOTAL);
    p[0x73] = h; p[0x74] = l;
    p[0x75] = 0x02;             // VECTOR_DMP_SET_PROPERTY
    p[0x76] = 0xA1;             // Address Type & Data Type
    // First Property Address = 0, Address Increment = 1
    p[0x79] = 0x00; p[0x7A] = 0x01;
    // Property Count = 513 (start code + 512 channels)
    p[0x7B] = 0x02; p[0x7C] = 0x01;
    // Start code 0x00 (null / standard DMX)
    p[0x7D] = 0x00;
    // DMX data
    p[0x7E..0x27E].copy_from_slice(data);

    p
}

/// Parse an E1.31 Data Packet.  Returns `(universe, priority, dmx_data)` or `None`.
pub fn parse_sacn(buf: &[u8]) -> Option<(u16, u8, &[u8])> {
    if buf.len() < 0x7E {
        return None;
    }
    if &buf[4..16] != ACN_ID {
        return None;
    }
    // Root vector must be VECTOR_ROOT_E131_DATA
    if &buf[0x12..0x16] != &[0x00, 0x00, 0x00, 0x04] {
        return None;
    }
    // Framing vector must be VECTOR_E131_DATA_PACKET
    if &buf[0x28..0x2C] != &[0x00, 0x00, 0x00, 0x02] {
        return None;
    }
    // DMP vector
    if buf[0x75] != 0x02 {
        return None;
    }
    // Only accept null start code
    if buf[0x7D] != 0x00 {
        return None;
    }
    let priority = buf[0x6C];
    let universe = ((buf[0x71] as u16) << 8) | (buf[0x72] as u16);
    if universe == 0 || universe > 63999 {
        return None;
    }
    let end = buf.len().min(0x7E + 512);
    Some((universe, priority, &buf[0x7E..end]))
}

/// Multicast address for a sACN universe: 239.255.(universe>>8).(universe&0xFF).
pub fn multicast_addr(universe: u16) -> Ipv4Addr {
    Ipv4Addr::new(239, 255, (universe >> 8) as u8, (universe & 0xFF) as u8)
}

// ─── Received sACN packet ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ReceivedSacnPacket {
    pub universe: u16,
    pub priority: u8,
    pub source: IpAddr,
    /// Fixed-size DMX data — avoids heap allocation on the UDP hot path.
    pub data: [u8; 512],
}

// ─── Bevy Resource ────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct SacnState {
    pub socket: Option<UdpSocket>,
    pub rx_chan: Option<Receiver<ReceivedSacnPacket>>,
    rx_thread_tx: Option<Sender<ReceivedSacnPacket>>,
    sequence: u8,
    /// Shared drop counter incremented by the RX thread when the channel is full.
    pub rx_drops: Arc<AtomicU64>,
    /// Signal for the RX background thread to exit.
    shutdown: Arc<AtomicBool>,
    /// Throttle bind retries to avoid log-spam when the port is busy.
    last_bind_attempt: Option<std::time::Instant>,
}

impl Default for SacnState {
    fn default() -> Self {
        Self {
            socket: None,
            rx_chan: None,
            rx_thread_tx: None,
            sequence: 0,
            rx_drops: Arc::new(AtomicU64::new(0)),
            shutdown: Arc::new(AtomicBool::new(false)),
            last_bind_attempt: None,
        }
    }
}

// ─── Systems ──────────────────────────────────────────────────────────────────

/// Manage sACN socket lifetime (port 5568, separate from Art-Net).
pub fn sacn_manage_socket(
    mut state: ResMut<SacnState>,
    cfg: Res<SacnConfig>,
    supervisor: Res<IoSupervisor>,
) {
    let wants_io = cfg.tx_enabled || cfg.rx_enabled;

    if wants_io && state.socket.is_none() {
        let now = std::time::Instant::now();
        let should_try = state.last_bind_attempt.map_or(true, |t| {
            now.duration_since(t).as_secs_f32() >= 1.0
        });
        if should_try {
            state.last_bind_attempt = Some(now);
            let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), SACN_PORT);
            match UdpSocket::bind(bind_addr) {
                Ok(sock) => {
                    sock.set_nonblocking(true).ok();
                    sock.set_broadcast(true).ok();
                    info!("sACN socket bound to {}", bind_addr);
                    state.socket = Some(sock);
                }
                Err(e) => warn!("sACN bind failed: {e}"),
            }
        }
    }

    if cfg.rx_enabled && state.rx_chan.is_none() {
        if let Some(ref sock) = state.socket {
            match sock.try_clone() {
                Ok(rx_sock) => {
                    rx_sock.set_nonblocking(false).ok();
                    rx_sock.set_read_timeout(Some(Duration::from_millis(100))).ok();

                    let (tx, rx) = bounded::<ReceivedSacnPacket>(8);
                    state.rx_chan = Some(rx);
                    state.rx_thread_tx = Some(tx.clone());
                    let drops = state.rx_drops.clone();
                    let shutdown = Arc::clone(&state.shutdown);

                    std::thread::spawn(move || {
                        let mut buf = [0u8; 700];
                        loop {
                            match rx_sock.recv_from(&mut buf) {
                                Ok((n, src)) => {
                                    if let Some((universe, priority, data)) =
                                        parse_sacn(&buf[..n])
                                    {
                                        let mut pkt_data = [0u8; 512];
                                        let len = data.len().min(512);
                                        pkt_data[..len].copy_from_slice(&data[..len]);
                                        let pkt = ReceivedSacnPacket {
                                            universe,
                                            priority,
                                            source: src.ip(),
                                            data: pkt_data,
                                        };
                                        if let Err(TrySendError::Full(_)) = tx.try_send(pkt) {
                                            drops.fetch_add(1, Ordering::Relaxed);
                                        }
                                    }
                                }
                                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                                    if shutdown.load(Ordering::Relaxed) {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    warn!("sACN RX error: {e}");
                                    std::thread::sleep(Duration::from_millis(10));
                                }
                            }
                        }
                    });
                }
                Err(e) => warn!("sACN RX socket clone failed: {e}"),
            }
        }
    }

    if !cfg.rx_enabled {
        state.shutdown.store(true, Ordering::Relaxed);
        state.rx_chan = None;
        state.rx_thread_tx = None;
        state.shutdown = Arc::new(AtomicBool::new(false));
    }

    // Release socket when both TX and RX are disabled.
    if !wants_io {
        state.socket = None;
    }

    // Sync thread-local drops into the supervisor.
    let local_drops = state.rx_drops.load(Ordering::Relaxed);
    let global = supervisor.rx_drops.load(Ordering::Relaxed);
    if local_drops > global {
        supervisor.rx_drops.store(local_drops, Ordering::Relaxed);
    }
}

/// Drain incoming sACN packets into the DMX engine "sacn_in" source.
pub fn sacn_receive(
    state: Res<SacnState>,
    mut engine: ResMut<DmxEngineRes>,
    mut stats: ResMut<SacnStats>,
) {
    let Some(rx) = &state.rx_chan else { return };

    // Use each packet's own priority for the merge source so higher-priority
    // sACN sources win. For simplicity we use a single fixed source here;
    // per-CID sources are a future enhancement.
    let source = engine
        .0
        .get_or_add_source("sacn_in", 100, stagelx_dmx::merge::MergeStrategy::Htp);

    let current_count = source.universes.universes().count();
    let mut new_count = current_count;
    let mut count = 0u64;

    while let Ok(pkt) = rx.try_recv() {
        let already_exists = source.universes.get(pkt.universe).is_some();
        if !already_exists && new_count >= MAX_RX_UNIVERSES {
            continue;
        }
        if !already_exists {
            new_count += 1;
        }
        let buf = source.universes.get_or_insert(pkt.universe);
        buf.copy_from_slice(&pkt.data);
        count += 1;
    }
    stats.rx_count = stats.rx_count.saturating_add(count);
}

/// Send sACN output for the configured universe.
/// Runs in FixedUpdate — no Instant throttle needed.
pub fn sacn_send(
    mut state: ResMut<SacnState>,
    engine: Res<DmxEngineRes>,
    cfg: Res<SacnConfig>,
    mut stats: ResMut<SacnStats>,
) {
    if !cfg.tx_enabled {
        return;
    }

    if state.socket.is_none() {
        return;
    }

    let universe = cfg.out_universe;
    if let Some(dmx_buf) = engine.0.output_buffer(universe) {
        // Capture and advance sequence before borrowing socket.
        let seq = state.sequence;
        state.sequence = state.sequence.wrapping_add(1);

        let pkt = build_sacn(universe, cfg.priority, seq, dmx_buf.as_bytes());

        let dest_ip: IpAddr = if cfg.dest_ip.trim().is_empty() {
            IpAddr::V4(multicast_addr(universe))
        } else {
            cfg.dest_ip
                .trim()
                .parse()
                .unwrap_or_else(|_| IpAddr::V4(multicast_addr(universe)))
        };
        let dest = SocketAddr::new(dest_ip, SACN_PORT);

        let sock = state.socket.as_ref().unwrap();
        match sock.send_to(&pkt, dest) {
            Ok(_) => {
                stats.tx_count = stats.tx_count.saturating_add(1);
                stats.status = ProtocolStatus::Live;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_e) => {
                stats.status = ProtocolStatus::Error;
            }
        }
    }
}
