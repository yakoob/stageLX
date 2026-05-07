use crate::error::GdtfError;

/// Parsed representation of a GDTF fixture type.
#[derive(Debug, Clone)]
pub struct GdtfFixtureType {
    /// GDTF FixtureTypeId GUID (e.g. "1234-5678-...").
    pub fixture_type_id: String,
    pub name: String,
    pub short_name: String,
    pub manufacturer: String,
    pub description: String,
    pub dmx_modes: Vec<DmxMode>,
    pub geometries: Vec<Geometry>,
    pub wheels: Vec<Wheel>,
}

#[derive(Debug, Clone)]
pub struct DmxMode {
    pub name: String,
    pub channels: Vec<DmxChannel>,
}

#[derive(Debug, Clone)]
pub struct DmxChannel {
    /// 1-based offset from fixture base address.
    pub offset: u16,
    /// GDTF attribute name (e.g. "Pan", "Dimmer", "ColorAdd_R").
    pub attribute: String,
    pub default_value: u8,
    pub resolution: u8,
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub name: String,
    pub geometry_type: GeometryType,
    pub children: Vec<Geometry>,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub enum GeometryType {
    General,
    Body,
    Yoke,
    Head,
    Beam { beam_angle: f32, beam_type: BeamType },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BeamType {
    Wash,
    Spot,
    None,
}

#[derive(Debug, Clone)]
pub struct Wheel {
    pub name: String,
    pub slots: Vec<WheelSlot>,
}

#[derive(Debug, Clone)]
pub struct WheelSlot {
    pub name: String,
    /// CIE xy + Y colour, if this is a colour slot.
    pub color: Option<[f32; 3]>,
    /// Path inside the GDTF ZIP to the gobo image asset.
    pub media_file: Option<String>,
}

/// Parse a raw GDTF file (ZIP archive bytes) into a [`GdtfFixtureType`].
pub fn parse_gdtf(_data: &[u8]) -> Result<GdtfFixtureType, GdtfError> {
    // Phase 1: implement ZIP extraction + description.xml parsing
    todo!("parse_gdtf: Phase 1 implementation")
}
