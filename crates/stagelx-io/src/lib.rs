pub mod artnet;
pub mod error;
pub mod midi;
pub mod osc;
pub mod sacn;
pub mod usb;

use bevy::prelude::*;
use artnet::{
    ArtNetState, DmxEngineRes,
    artnet_manage_socket, artnet_receive, artnet_send,
    dmx_engine_tick, programmer_to_dmx,
};
use sacn::{SacnState, sacn_manage_socket, sacn_receive, sacn_send};
use stagelx_dmx::engine::DmxEngine;

/// Art-Net and sACN output rate.  E1.31 §6.6 recommends ≥ 44 Hz; Art-Net
/// nodes also expect keepalives at this rate.
const DMX_OUTPUT_HZ: f64 = 44.0;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DmxEngineRes(DmxEngine::default()))
            .init_resource::<ArtNetState>()
            .init_resource::<SacnState>()
            // Run the fixed-rate DMX output at exactly 44 Hz regardless of
            // render frame rate.  Instant-polling in Update would work but
            // burns CPU checking the clock on every rendered frame.
            .insert_resource(Time::<Fixed>::from_hz(DMX_OUTPUT_HZ))
            // Every render frame: manage sockets and drain incoming packets.
            .add_systems(
                Update,
                (
                    artnet_manage_socket,
                    sacn_manage_socket,
                    artnet_receive,
                    sacn_receive,
                )
                    .chain(),
            )
            // Exactly 44 times/sec: write programmer → DMX, merge, send.
            .add_systems(
                FixedUpdate,
                (programmer_to_dmx, dmx_engine_tick, artnet_send, sacn_send).chain(),
            );
    }
}
