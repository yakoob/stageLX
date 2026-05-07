use crate::error::GdtfError;
use stagelx_core::fixture::FixtureInstance;

/// A parsed MVR scene: fixtures with positions, embedded GDTF files, and venue geometry.
#[derive(Debug, Default)]
pub struct MvrScene {
    pub name: String,
    pub fixture_instances: Vec<FixtureInstance>,
    /// Raw bytes of each embedded GDTF file, keyed by filename.
    pub gdtf_files: Vec<(String, Vec<u8>)>,
    /// Optional venue geometry file paths (OBJ/glTF inside the MVR ZIP).
    pub geometry_files: Vec<String>,
}

/// Parse a raw MVR file (ZIP archive bytes) into an [`MvrScene`].
pub fn parse_mvr(_data: &[u8]) -> Result<MvrScene, GdtfError> {
    // Phase 4: implement MVR parser (GeneralSceneDescription.xml + embedded GDTFs)
    todo!("parse_mvr: Phase 4 implementation")
}
