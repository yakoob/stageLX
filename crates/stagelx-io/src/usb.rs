//! USB DMX output via the Enttec USB Pro protocol.
//!
//! The Enttec USB Pro presents as a virtual COM port (FTDI chip, VID 0403).
//! Protocol: UART framing at 250 000 baud.
//!
//! Output frame (label 6 — "Output Only Send DMX Packet Request"):
//!   0x7E  label  len_lsb  len_msb  start_code  dmx[0..512]  0xE7
//!
//! Total frame = 518 bytes; baud-rate gives ≈ 22 ms/frame ≈ 45 Hz max.
//!
//! TX runs in a background thread (IoSink) so the 16–22 ms serial write does
//! not stall Bevy's FixedUpdate tick.

use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use bevy::prelude::*;
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use stagelx_dmx::engine::DmxEngineRes;
use stagelx_show::ProtocolStatus;
use crate::config::UsbConfig;
use crate::stats::UsbStats;
use crate::supervisor::{IoSink, IoSupervisor};

pub const ENTTEC_BAUD: u32 = 250_000;
const LABEL_OUTPUT_DMX: u8 = 6;
const DMX_PAYLOAD: u16 = 513; // null start code + 512 channels

// ─── TX Command ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UsbTxCmd {
    pub data: [u8; 512],
}

// ─── IoSink implementation ────────────────────────────────────────────────────

pub struct UsbTxSink {
    port: String,
    baud: u32,
}

impl UsbTxSink {
    pub fn new(port: String, baud: u32) -> Self {
        Self { port, baud }
    }
}

impl IoSink for UsbTxSink {
    type Cmd = UsbTxCmd;

    fn start(&self, rx: Receiver<Self::Cmd>, shutdown: Receiver<()>) -> std::io::Result<JoinHandle<()>> {
        let port = self.port.clone();
        let baud = self.baud;

        let mut dev = serialport::new(&port, baud)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        Ok(std::thread::spawn(move || {
            loop {
                crossbeam_channel::select! {
                    recv(rx) -> cmd => {
                        match cmd {
                            Ok(cmd) => {
                                let frame = build_enttec_frame(&cmd.data);
                                if let Err(e) = dev.write_all(&frame) {
                                    warn!("USB DMX write error: {e}");
                                    // Try to re-open the device once.
                                    if let Ok(d) = serialport::new(&port, baud)
                                        .timeout(Duration::from_millis(100))
                                        .open()
                                    {
                                        dev = d;
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    recv(shutdown) -> _ => break,
                }
            }
        }))
    }
}

// ─── Resource ─────────────────────────────────────────────────────────────────

/// USB state is now Send-safe because the SerialPort lives in the background thread.
#[derive(Resource)]
pub struct UsbDmxState {
    pub tx_chan: Option<Sender<UsbTxCmd>>,
    tx_shutdown: Option<Sender<()>>,
    tx_handle: Option<JoinHandle<()>>,
    pub tx_drops: Arc<AtomicU64>,
    /// Last port path we successfully opened (for re-open logic).
    last_port: String,
}

impl Default for UsbDmxState {
    fn default() -> Self {
        Self {
            tx_chan: None,
            tx_shutdown: None,
            tx_handle: None,
            tx_drops: Arc::new(AtomicU64::new(0)),
            last_port: String::new(),
        }
    }
}

// ─── Systems ──────────────────────────────────────────────────────────────────

/// Open or close the USB serial device based on `UsbConfig`.
/// Runs in `Update` so it can respond quickly to config changes.
pub fn usb_manage_device(
    mut state: ResMut<UsbDmxState>,
    cfg: ResMut<UsbConfig>,
    mut stats: ResMut<UsbStats>,
    supervisor: Res<IoSupervisor>,
) {
    let port = cfg.port.trim();

    // ── Start TX thread when enabled ──────────────────────────────────────────
    if cfg.tx_enabled && state.tx_chan.is_none() {
        if port.is_empty() {
            stats.status = ProtocolStatus::Warn;
            return;
        }
        let (tx, rx) = bounded::<UsbTxCmd>(1);
        let (shutdown_tx, shutdown_rx) = bounded::<()>(1);
        let sink = UsbTxSink::new(port.to_string(), ENTTEC_BAUD);
        match sink.start(rx, shutdown_rx) {
            Ok(handle) => {
                info!("USB DMX TX thread started: {}", port);
                stats.status = ProtocolStatus::Live;
                state.tx_chan = Some(tx);
                state.tx_shutdown = Some(shutdown_tx);
                state.tx_handle = Some(handle);
                state.last_port = port.to_string();
            }
            Err(_e) => {
                stats.status = ProtocolStatus::Error;
            }
        }
    }

    // ── Stop TX thread when disabled ──────────────────────────────────────────
    if !cfg.tx_enabled && state.tx_chan.is_some() {
        if let Some(shutdown) = state.tx_shutdown.take() {
            let _ = shutdown.try_send(());
        }
        state.tx_chan = None;
        state.tx_handle = None;
        stats.status = ProtocolStatus::Idle;
        info!("USB DMX TX thread stopped");
    }

    // ── Sync tx_drops into supervisor ─────────────────────────────────────────
    let local_drops = state.tx_drops.load(Ordering::Relaxed);
    let global = supervisor.tx_drops.load(Ordering::Relaxed);
    if local_drops > global {
        supervisor.tx_drops.store(local_drops, Ordering::Relaxed);
    }
}

/// Send a DMX frame over the Enttec USB Pro device.
/// Runs in `FixedUpdate` at 44 Hz — queues a command for the TX background thread.
pub fn usb_send(
    state: Res<UsbDmxState>,
    engine: Res<DmxEngineRes>,
    cfg: Res<UsbConfig>,
    mut stats: ResMut<UsbStats>,
) {
    if !cfg.tx_enabled {
        return;
    }
    let Some(tx) = &state.tx_chan else { return };

    let universe = cfg.universe;
    if let Some(dmx_buf) = engine.0.output_buffer(universe) {
        let cmd = UsbTxCmd {
            data: *dmx_buf.as_bytes(),
        };
        match tx.try_send(cmd) {
            Ok(_) => {
                stats.tx_count = stats.tx_count.saturating_add(1);
                stats.last_tx_at = Some(std::time::Instant::now());
                stats.status = ProtocolStatus::Live;
            }
            Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => {
                stats.status = ProtocolStatus::Error;
            }
        }
    }
}

// ─── Frame builder ────────────────────────────────────────────────────────────

fn build_enttec_frame(data: &[u8; 512]) -> [u8; 518] {
    let mut frame = [0u8; 518];
    frame[0] = 0x7E;                            // Start delimiter
    frame[1] = LABEL_OUTPUT_DMX;
    frame[2] = (DMX_PAYLOAD & 0xFF) as u8;      // Length LSB (513 = 0x01)
    frame[3] = ((DMX_PAYLOAD >> 8) & 0xFF) as u8; // Length MSB (513 >> 8 = 0x02)
    frame[4] = 0x00;                            // DMX null start code
    frame[5..517].copy_from_slice(data);
    frame[517] = 0xE7;                          // End delimiter
    frame
}
