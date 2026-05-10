//! MVR (My Virtual Rig) export — ANSI E1.54.
//!
//! Generates a ZIP archive containing:
//! - `GeneralSceneDescription.xml` — fixture list with 3D transforms and DMX addresses
//! - One embedded `.gdtf` fixture file per unique fixture type referenced by the patch
//!
//! Coordinate system: Bevy (Y-up, metres) → MVR (Z-up, mm).
//!   MVR.X =  Bevy.X * 1000
//!   MVR.Y = -Bevy.Z * 1000
//!   MVR.Z =  Bevy.Y * 1000

use std::collections::HashSet;
use std::io::{Cursor, Write};
use zip::{write::SimpleFileOptions, ZipWriter};

use stagelx_core::fixture::FixtureInstance;
use crate::error::GdtfError;

/// Export the current patch as an MVR file (in-memory ZIP bytes).
///
/// `resolve_gdtf` must return the original GDTF ZIP bytes for a given fixture type ID.
/// `scene_name` is written into the XML and used as the MVR filename hint.
pub fn export_mvr(
    fixtures: &[&FixtureInstance],
    resolve_gdtf: impl Fn(&str) -> Option<Vec<u8>>,
    scene_name: &str,
) -> Result<Vec<u8>, GdtfError> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Collect unique fixture types and their GDTF bytes.
    let mut type_ids = HashSet::new();
    let mut gdtf_entries: Vec<(String, Vec<u8>)> = Vec::new();

    for inst in fixtures {
        if type_ids.insert(&inst.fixture_type_id) {
            if let Some(bytes) = resolve_gdtf(&inst.fixture_type_id) {
                gdtf_entries.push((gdtf_filename(inst, &bytes), bytes));
            }
        }
    }

    // Write GeneralSceneDescription.xml.
    let xml = build_scene_xml(fixtures, scene_name, &gdtf_entries);
    zip.start_file("GeneralSceneDescription.xml", options)?;
    zip.write_all(xml.as_bytes())?;

    // Embed GDTF files.
    for (filename, bytes) in &gdtf_entries {
        zip.start_file(filename, options)?;
        zip.write_all(bytes)?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

// ─── XML generation ───────────────────────────────────────────────────────────

fn build_scene_xml(
    fixtures: &[&FixtureInstance],
    scene_name: &str,
    gdtf_entries: &[(String, Vec<u8>)],
) -> String {
    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str("<GeneralSceneDescription>\n");
    xml.push_str("  <Scene>\n");
    if !scene_name.is_empty() {
        xml.push_str(&format!("    <Name>{}</Name>\n", xml_escape(scene_name)));
    }
    xml.push_str("    <Fixtures>\n");

    for inst in fixtures {
        let gdtf_spec = gdtf_entries
            .iter()
            .find(|(_, bytes)| match_gdtf_for_instance(inst, bytes))
            .map(|(name, _)| name.as_str())
            .unwrap_or(&inst.fixture_type_id);

        xml.push_str(&format!(
            r#"      <Fixture name="{}">"#,
            xml_escape(&inst.name)
        ));
        xml.push('\n');
        xml.push_str(&format!("        <GDTFSpec>{}</GDTFSpec>\n", xml_escape(gdtf_spec)));
        xml.push_str(&format!(
            "        <GDTFMode>{}</GDTFMode>\n",
            xml_escape(&inst.dmx_mode)
        ));
        xml.push_str(&format!(
            "        <FixtureTypeId>{}</FixtureTypeId>\n",
            xml_escape(&inst.fixture_type_id)
        ));
        xml.push_str(&format!(
            "        <Matrix>{}</Matrix>\n",
            build_matrix(inst.position, inst.rotation)
        ));
        xml.push_str(&format!(
            "        <Address>{}.{}</Address>\n",
            inst.address.universe, inst.address.channel
        ));
        xml.push_str("      </Fixture>\n");
    }

    xml.push_str("    </Fixtures>\n");
    xml.push_str("  </Scene>\n");
    xml.push_str("</GeneralSceneDescription>\n");
    xml
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Generate a safe filename for an embedded GDTF.
/// Falls back to the fixture type ID if no bytes are available.
fn gdtf_filename(inst: &FixtureInstance, _bytes: &[u8]) -> String {
    // Sanitize manufacturer + name into a filename.
    let safe = sanitize_filename(&format!("{}_{}", inst.fixture_type_id, inst.dmx_mode));
    format!("{}.gdtf", safe)
}

/// Best-effort check whether a GDTF byte blob belongs to a fixture instance.
/// We match by fixture type ID string occurrence in the ZIP comment / XML header.
fn match_gdtf_for_instance(inst: &FixtureInstance, bytes: &[u8]) -> bool {
    // Fast path: scan the first 4 KB of the ZIP for the fixture type ID.
    let scan_len = bytes.len().min(4096);
    let haystack = &bytes[..scan_len];
    // Simple substring search — adequate because the description.xml inside
    // the GDTF ZIP contains the FixtureTypeID attribute.
    find_bytes(haystack, inst.fixture_type_id.as_bytes()).is_some()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            ' ' | '\t' => '_',
            _ => '_',
        })
        .collect()
}

// ─── Matrix conversion ────────────────────────────────────────────────────────

/// Build the MVR 4×4 column-major matrix string from Bevy position and rotation.
///
/// Bevy:  position = [x, y, z] metres, rotation = [roll, yaw, pitch] degrees (ZYX euler).
/// MVR:   column-major {u0..u3, v0..v3, w0..w3, o0..o3} with translation in mm.
fn build_matrix(position: [f32; 3], rotation: [f32; 3]) -> String {
    // 1. Reconstruct Bevy rotation matrix rb from ZYX euler angles.
    let r = rotation[0].to_radians(); // roll  (about X)
    let y = rotation[1].to_radians(); // yaw   (about Y)
    let p = -rotation[2].to_radians(); // pitch (about Z) — negate because parser's pitch = -actual

    let (cr, sr) = (r.cos(), r.sin());
    let (cy, sy) = (y.cos(), y.sin());
    let (cp, sp) = (p.cos(), p.sin());

    let rb = [
        [cy * cp, cy * sp * sr - sy * cr, cy * sp * cr + sy * sr],
        [sy * cp, sy * sp * sr + cy * cr, sy * sp * cr - cy * sr],
        [-sp, cp * sr, cp * cr],
    ];

    // 2. Convert rb → MVR rotation matrix r_mvr.
    //    Forward (parser):  rb[i][j] = T[i][k] * r[k][l] * T[l][j]
    //    Inverse (export):  r_mvr = T^T * rb * T
    let rm = [
        [rb[0][0], -rb[0][2], rb[0][1]],
        [-rb[2][0], rb[2][2], -rb[2][1]],
        [rb[1][0], -rb[1][2], rb[1][1]],
    ];

    // 3. Translation: metres → mm, swap axes.
    let tx = position[0] * 1000.0;
    let ty = -position[2] * 1000.0;
    let tz = position[1] * 1000.0;

    // 4. Column-major 4×4: {u, v, w, o}
    format!(
        "{{{:.6},{:.6},{:.6},0,{:.6},{:.6},{:.6},0,{:.6},{:.6},{:.6},0,{:.6},{:.6},{:.6},1}}",
        rm[0][0], rm[1][0], rm[2][0], // column 0 (u)
        rm[0][1], rm[1][1], rm[2][1], // column 1 (v)
        rm[0][2], rm[1][2], rm[2][2], // column 2 (w)
        tx, ty, tz,                   // column 3 (o)
    )
}

/// Naïve byte-level substring search.
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_identity_matrix() {
        // Zero rotation, known position.
        let pos = [1.0, 2.0, 3.0];
        let rot = [0.0, 0.0, 0.0];
        let s = build_matrix(pos, rot);
        // Should contain the translation terms.
        assert!(s.contains("1000"));   // tx
        assert!(s.contains("-3000"));  // ty = -position[2]*1000
        assert!(s.contains("2000"));   // tz = position[1]*1000
    }

    #[test]
    fn xml_escaping() {
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("\"quote\""), "&quot;quote&quot;");
    }
}
