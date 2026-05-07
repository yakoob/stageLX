use serde::{Deserialize, Serialize};

/// Physical value for a single GDTF attribute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    /// Raw 8-bit DMX byte (fallback when no physical mapping is known).
    Raw(u8),
    /// Normalized 0.0–1.0 (Dimmer, Iris, Zoom, etc.).
    Ratio(f32),
    /// Rotation in degrees (Pan, Tilt, GoboRotation, etc.).
    Angle(f32),
    /// Linear RGB, each component 0.0–1.0.
    Color([f32; 3]),
    /// Wheel slot index (0-based).
    SlotIndex(u32),
}

/// Describes a single controllable attribute on a fixture type, derived from GDTF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    /// GDTF attribute name, e.g. "Pan", "Dimmer", "ColorAdd_R".
    pub name: String,
    /// 1-based offset from the fixture's base DMX address.
    pub dmx_offset: u16,
    /// Number of DMX bytes (1 = 8-bit, 2 = 16-bit).
    pub resolution: u8,
    pub default_value: u8,
}
