//! Performance diagnostics collection.
//!
//! Systems here populate `PerfDiagnosticsRes` so the UI can display a live HUD.

use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::mesh::Indices;
use bevy::render::render_resource::VertexFormat;

use crate::{
    fixture::BeamCone,
    lod::BeamLodTier,
};

// ─── Systems ──────────────────────────────────────────────────────────────────

/// Update frame time from Bevy's diagnostics store.
pub fn track_frame_time(
    diagnostics: Res<DiagnosticsStore>,
    mut perf: ResMut<stagelx_show::PerfDiagnosticsRes>,
) {
    if let Some(diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(m) = diag.measurements().last() {
            perf.frame_time_ms = m.value as f32;
        }
    }
}

/// Count total beams and ray-marched beams (Tier 1 + Tier 2).
pub fn track_beam_counts(
    all_beams: Query<(), With<BeamCone>>,
    raymarched: Query<(), (With<BeamCone>, With<BeamLodTier>)>,
    mut perf: ResMut<stagelx_show::PerfDiagnosticsRes>,
) {
    perf.beam_count = all_beams.iter().count();
    perf.beam_raymarch_count = raymarched.iter().count();
}

/// Estimate GPU memory for all meshes in the scene.
///
/// Computes vertex + index buffer sizes from `Assets<Mesh>` for every
/// `Mesh3d` handle found in the world. Sizes are cached in a `Local` so
/// that after meshes are extracted to `RenderWorld` we still report the
/// last-known value instead of panicking.
pub fn estimate_gpu_memory(
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Mesh3d>,
    mut perf: ResMut<stagelx_show::PerfDiagnosticsRes>,
    mut cache: Local<std::collections::HashMap<AssetId<Mesh>, usize>>,
) {
    let mut total_bytes: usize = 0;

    for mesh3d in &mesh_query {
        let id = mesh3d.0.id();

        // Try to read the mesh directly (works before extraction).
        if let Some(mesh) = meshes.get(&mesh3d.0) {
            let size = compute_mesh_size(mesh);
            cache.insert(id, size);
            total_bytes += size;
        } else if let Some(&cached) = cache.get(&id) {
            // Mesh has been extracted to RenderWorld — use cached size.
            total_bytes += cached;
        }
    }

    perf.estimated_gpu_memory_mb = (total_bytes as f32) / (1024.0 * 1024.0);
}

fn compute_mesh_size(mesh: &Mesh) -> usize {
    // `try_attributes` is safe — it returns Err when the mesh has been
    // extracted to RenderWorld instead of panicking like `attributes()`.
    let vertex_bytes = match mesh.try_attributes() {
        Ok(attrs) => attrs
            .map(|(_, values)| values.len() * VertexFormat::from(values).size() as usize)
            .sum(),
        Err(_) => 0,
    };

    // `try_indices()` is safe — `indices()` also panics on extracted meshes.
    let index_bytes = match mesh.try_indices() {
        Ok(Indices::U16(v)) => v.len() * 2,
        Ok(Indices::U32(v)) => v.len() * 4,
        _ => 0,
    };

    vertex_bytes + index_bytes
}
