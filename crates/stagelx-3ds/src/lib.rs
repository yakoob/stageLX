//! Pure-Rust parser for Autodesk 3D Studio binary files (.3ds).
//!
//! Produces a flat list of named triangle meshes with vertex positions,
//! face indices, and optional UV coordinates.  All other 3DS data
//! (materials, keyframes, lights, cameras) is silently skipped.
//!
//! Coordinate system: 3DS uses right-handed Y-up, matching Bevy's convention,
//! so vertex data can be used without axis permutation.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error3ds {
    #[error("data too short to be a valid 3DS file")]
    Truncated,
    #[error("not a 3DS file — missing MAIN3DS header (0x4D4D)")]
    NotA3ds,
}

/// A single named triangle mesh extracted from a .3ds file.
#[derive(Debug, Default, Clone)]
pub struct Mesh3ds {
    pub name:     String,
    pub vertices: Vec<[f32; 3]>,
    pub faces:    Vec<[u16; 3]>,
    pub uvs:      Vec<[f32; 2]>,
}

/// A parsed .3ds scene — one or more named meshes.
#[derive(Debug, Default)]
pub struct Scene3ds {
    pub meshes: Vec<Mesh3ds>,
}

// ─── Public entry point ───────────────────────────────────────────────────────

pub fn parse(data: &[u8]) -> Result<Scene3ds, Error3ds> {
    if data.len() < 6 {
        return Err(Error3ds::Truncated);
    }
    let id  = u16::from_le_bytes([data[0], data[1]]);
    if id != 0x4D4D {
        return Err(Error3ds::NotA3ds);
    }
    let len = u32::from_le_bytes([data[2], data[3], data[4], data[5]]) as usize;
    let end = len.min(data.len());

    let mut scene = Scene3ds::default();
    parse_main(&data[6..end], &mut scene);
    Ok(scene)
}

// ─── Chunk walker ─────────────────────────────────────────────────────────────

/// Iterate over all top-level chunks in `data`, calling `f(chunk_id, chunk_data)`.
/// `chunk_data` excludes the 6-byte header; malformed chunks are skipped silently.
fn walk_chunks(data: &[u8], mut f: impl FnMut(u16, &[u8])) {
    let mut pos = 0;
    while pos + 6 <= data.len() {
        let id  = u16::from_le_bytes([data[pos], data[pos + 1]]);
        let len = u32::from_le_bytes([
            data[pos + 2], data[pos + 3],
            data[pos + 4], data[pos + 5],
        ]) as usize;
        if len < 6 {
            break; // malformed: minimum chunk is 6 bytes (header only)
        }
        let end = (pos + len).min(data.len());
        f(id, &data[pos + 6..end]);
        pos += len;
    }
}

// ─── Main → Edit3DS ───────────────────────────────────────────────────────────

fn parse_main(data: &[u8], scene: &mut Scene3ds) {
    walk_chunks(data, |id, cd| {
        if id == 0x3D3D {
            parse_edit3ds(cd, scene);
        }
    });
}

fn parse_edit3ds(data: &[u8], scene: &mut Scene3ds) {
    walk_chunks(data, |id, cd| {
        if id == 0x4000 {
            if let Some(mesh) = parse_named_object(cd) {
                if !mesh.vertices.is_empty() {
                    scene.meshes.push(mesh);
                }
            }
        }
    });
}

// ─── Named Object → Triangle Mesh ─────────────────────────────────────────────

fn parse_named_object(data: &[u8]) -> Option<Mesh3ds> {
    // Data starts with a null-terminated object name.
    let name_end = data.iter().position(|&b| b == 0)?;
    let name = String::from_utf8_lossy(&data[..name_end]).into_owned();
    let rest = data.get((name_end + 1)..).unwrap_or(&[]);

    let mut mesh = Mesh3ds { name, ..Default::default() };
    walk_chunks(rest, |id, cd| {
        if id == 0x4100 {
            parse_tri_mesh(cd, &mut mesh);
        }
    });
    Some(mesh)
}

fn parse_tri_mesh(data: &[u8], mesh: &mut Mesh3ds) {
    walk_chunks(data, |id, cd| match id {
        0x4110 => parse_point_array(cd, mesh),
        0x4120 => parse_face_array(cd, mesh),
        0x4140 => parse_tex_verts(cd, mesh),
        _ => {}
    });
}

// ─── Geometry sub-chunks ──────────────────────────────────────────────────────

fn parse_point_array(data: &[u8], mesh: &mut Mesh3ds) {
    if data.len() < 2 { return; }
    let count = u16::from_le_bytes([data[0], data[1]]) as usize;
    if data.len() < 2 + count * 12 { return; }
    mesh.vertices.reserve(count);
    for i in 0..count {
        let o = 2 + i * 12;
        mesh.vertices.push([
            f32::from_le_bytes(data[o..o + 4].try_into().unwrap()),
            f32::from_le_bytes(data[o + 4..o + 8].try_into().unwrap()),
            f32::from_le_bytes(data[o + 8..o + 12].try_into().unwrap()),
        ]);
    }
}

fn parse_face_array(data: &[u8], mesh: &mut Mesh3ds) {
    if data.len() < 2 { return; }
    let count = u16::from_le_bytes([data[0], data[1]]) as usize;
    // Each face: 3 × u16 vertex indices + u16 flags = 8 bytes
    if data.len() < 2 + count * 8 { return; }
    mesh.faces.reserve(count);
    for i in 0..count {
        let o = 2 + i * 8;
        mesh.faces.push([
            u16::from_le_bytes([data[o],     data[o + 1]]),
            u16::from_le_bytes([data[o + 2], data[o + 3]]),
            u16::from_le_bytes([data[o + 4], data[o + 5]]),
        ]);
    }
}

fn parse_tex_verts(data: &[u8], mesh: &mut Mesh3ds) {
    if data.len() < 2 { return; }
    let count = u16::from_le_bytes([data[0], data[1]]) as usize;
    if data.len() < 2 + count * 8 { return; }
    mesh.uvs.reserve(count);
    for i in 0..count {
        let o = 2 + i * 8;
        mesh.uvs.push([
            f32::from_le_bytes(data[o..o + 4].try_into().unwrap()),
            f32::from_le_bytes(data[o + 4..o + 8].try_into().unwrap()),
        ]);
    }
}

// ─── Bevy mesh conversion ─────────────────────────────────────────────────────

/// Convert a [`Mesh3ds`] into the vertex + index data needed to build a Bevy
/// `Mesh`.  Returns `(positions, normals, uvs, indices)`.
///
/// Normals are computed per-face (flat shading) — adequate for fixture bodies.
pub fn to_bevy_buffers(
    mesh: &Mesh3ds,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals:   Vec<[f32; 3]> = Vec::new();
    let mut uvs:       Vec<[f32; 2]> = Vec::new();
    let mut indices:   Vec<u32>      = Vec::new();

    let has_uvs = mesh.uvs.len() == mesh.vertices.len();

    for face in &mesh.faces {
        let base = positions.len() as u32;
        for &vi in face.iter() {
            let vp = mesh.vertices.get(vi as usize).copied().unwrap_or([0.0; 3]);
            positions.push(vp);
            let uv = if has_uvs {
                mesh.uvs.get(vi as usize).copied().unwrap_or([0.0; 2])
            } else {
                [0.0; 2]
            };
            uvs.push(uv);
        }
        // Flat normal = cross product of two edges
        let [a, b, c] = [positions[base as usize], positions[base as usize + 1], positions[base as usize + 2]];
        let n = cross(sub(b, a), sub(c, a));
        let n = normalize(n);
        normals.push(n);
        normals.push(n);
        normals.push(n);
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
    }

    (positions, normals, uvs, indices)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt();
    if len < 1e-10 { return [0.0, 1.0, 0.0]; }
    [v[0]/len, v[1]/len, v[2]/len]
}
