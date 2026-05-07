use serde::{Deserialize, Serialize};
use crate::types::{DmxAddress, FixtureId};

/// A patched fixture instance in the show.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureInstance {
    pub id: FixtureId,
    /// Human-readable label shown in the UI.
    pub name: String,
    /// GDTF FixtureTypeId GUID, links to a loaded GdtfFixtureType.
    pub fixture_type_id: String,
    /// Active DMX mode name, must match a mode in the GDTF definition.
    pub dmx_mode: String,
    /// Starting DMX address for this instance.
    pub address: DmxAddress,
    /// 3D position in the scene (x, y, z) in metres.
    pub position: [f32; 3],
    /// Euler rotation in degrees (rx, ry, rz).
    pub rotation: [f32; 3],
}
