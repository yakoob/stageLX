use bevy::{
    camera::{RenderTarget, Viewport, visibility::RenderLayers},
    prelude::*,
    render::render_resource::{AsBindGroup, TextureFormat},
    shader::ShaderRef,
};

use stagelx_core::types::FixtureId;

use crate::{
    beam::BeamMaterial,
    beam_sprite::BeamSprite,
    fixture::BeamCone,
    scene::FohCamera,
};

// ─── Constants ────────────────────────────────────────────────────────────────

/// Screen-space radius threshold (px) between Tier 0 and Tier 1.
const TIER_1_THRESHOLD_PX: f32 = 50.0;
/// Screen-space radius threshold (px) between Tier 1 and Tier 2.
const TIER_2_THRESHOLD_PX: f32 = 200.0;
/// Maximum number of beams that may ray-march (Tier 1 + Tier 2).
const RAY_MARCH_HARD_CAP: usize = 64;

// ─── Components ───────────────────────────────────────────────────────────────

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BeamLodTier {
    /// Billboard sprite — zero ray-march cost.
    Tier0,
    /// Half-res offscreen ray march (16 steps).
    Tier1,
    /// Full-res ray march (32 steps).
    Tier2,
}

/// Marker for the half-resolution beam camera.
#[derive(Component)]
pub struct BeamHalfResCamera;

/// Marker for the fullscreen composite quad that adds the half-res beam pass.
#[derive(Component)]
pub struct BeamCompositeQuad;

// ─── Resources ────────────────────────────────────────────────────────────────

/// Owns the half-res render target handle so systems can reference it.
#[derive(Resource)]
pub struct BeamRenderTarget {
    pub half_res: Handle<Image>,
}

// ─── Composite material ───────────────────────────────────────────────────────

/// Fullscreen quad material that samples the half-res beam texture with additive blending.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BeamCompositeMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub beam_texture: Handle<Image>,
}

impl Material for BeamCompositeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/beam_composite.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

// ─── Startup: create render target, beam camera, composite quad ───────────────

pub fn setup_beam_lod(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BeamCompositeMaterial>>,
    windows: Query<&Window>,
) {
    let window = windows.single().expect("primary window");
    let (w, h) = (window.physical_width(), window.physical_height());
    let (half_w, half_h) = ((w / 2).max(1), (h / 2).max(1));

    let image = Image::new_target_texture(
        half_w,
        half_h,
        TextureFormat::Rgba16Float,
        None,
    );

    let half_res_handle: Handle<Image> = images.add(image);

    commands.insert_resource(BeamRenderTarget {
        half_res: half_res_handle.clone(),
    });

    // Half-res beam camera (layer 1 only).
    commands.spawn((
        BeamHalfResCamera,
        Camera3d::default(),
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            viewport: Some(Viewport {
                physical_position: UVec2::ZERO,
                physical_size: UVec2::new(half_w, half_h),
                depth: 0.0..1.0,
            }),
            ..default()
        },
        RenderTarget::Image(half_res_handle.clone().into()),
        RenderLayers::layer(1),
        Transform::default(),
    ));

    // Fullscreen composite quad (layer 0, main camera only).
    // Size is arbitrary — it will be rescaled each frame to fill the viewport.
    let quad_mesh = meshes.add(bevy::math::primitives::Rectangle::new(1.0, 1.0));
    let composite_mat = materials.add(BeamCompositeMaterial {
        beam_texture: half_res_handle,
    });

    commands.spawn((
        BeamCompositeQuad,
        Mesh3d(quad_mesh),
        MeshMaterial3d(composite_mat),
        Visibility::default(),
        RenderLayers::layer(0),
    ));
}

// ─── System: sync beam camera + composite quad to FOH camera ──────────────────

pub fn sync_beam_camera_to_foh(
    foh_q: Query<&Transform, With<FohCamera>>,
    mut beam_cam_q: Query<&mut Transform, (With<BeamHalfResCamera>, Without<FohCamera>)>,
    mut composite_q: Query<
        &mut Transform,
        (With<BeamCompositeQuad>, Without<FohCamera>, Without<BeamHalfResCamera>),
    >,
    windows: Query<&Window>,
    projection_q: Query<&Projection, With<FohCamera>>,
) {
    let Ok(foh_tf) = foh_q.single() else { return };
    let Ok(window) = windows.single() else { return };

    // Sync beam camera transform.
    for mut tf in &mut beam_cam_q {
        *tf = *foh_tf;
    }

    // Position composite quad just in front of the near plane.
    let near = 0.11; // slightly beyond default near plane
    let forward = foh_tf.forward();
    let pos = foh_tf.translation + forward * near;

    // Scale quad to exactly fill viewport at `near` distance.
    let aspect = if window.height() > 0.0 {
        window.width() / window.height()
    } else {
        16.0 / 9.0
    };

    // Extract FOV from projection.
    let fov_y = match projection_q.single() {
        Ok(Projection::Perspective(p)) => p.fov,
        _ => std::f32::consts::FRAC_PI_3,
    };

    let half_h = near * (fov_y * 0.5).tan();
    let half_w = half_h * aspect;

    for mut tf in &mut composite_q {
        tf.translation = pos;
        tf.rotation = foh_tf.rotation;
        tf.scale = Vec3::new(half_w * 2.0, half_h * 2.0, 1.0);
    }
}

// ─── System: evaluate LOD tiers per frame ─────────────────────────────────────

pub fn evaluate_beam_lod(
    windows: Query<&Window>,
    foh_q: Query<(&Transform, &Projection), With<FohCamera>>,
    beam_q: Query<(Entity, &GlobalTransform, &BeamCone)>,
    mut commands: Commands,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((foh_tf, foh_proj)) = foh_q.single() else { return };

    let fov_y = match foh_proj {
        Projection::Perspective(p) => p.fov,
        _ => std::f32::consts::FRAC_PI_3,
    };
    let vp_h = window.physical_height() as f32;

    // 1. Compute screen-space radius for every beam.
    let mut scored: Vec<(Entity, f32, BeamLodTier)> = Vec::with_capacity(beam_q.iter().len());
    for (entity, global_tf, _beam) in &beam_q {
        let pos = global_tf.translation();
        let dist = pos.distance(foh_tf.translation);
        if dist < 1e-3 {
            continue;
        }

        // Approximate base radius from beam cone scale.
        // The cone mesh has base radius proportional to its X/Z scale.
        let scale = global_tf.scale();
        let approx_base_radius = scale.x.max(scale.z) * 0.5;

        let angular_radius = (approx_base_radius / dist).atan();
        let pixel_radius = angular_radius / (fov_y * 0.5).tan() * (vp_h * 0.5);

        let tier = if pixel_radius < TIER_1_THRESHOLD_PX {
            BeamLodTier::Tier0
        } else if pixel_radius < TIER_2_THRESHOLD_PX {
            BeamLodTier::Tier1
        } else {
            BeamLodTier::Tier2
        };

        scored.push((entity, pixel_radius, tier));
    }

    // 2. Enforce hard cap: only top 64 by pixel radius may be Tier 1/2.
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut ray_march_count = 0usize;
    for (_entity, _radius, tier) in &mut scored {
        if *tier != BeamLodTier::Tier0 {
            if ray_march_count >= RAY_MARCH_HARD_CAP {
                *tier = BeamLodTier::Tier0;
            } else {
                ray_march_count += 1;
            }
        }
    }

    // 3. Write tier components.
    for (entity, _radius, tier) in scored {
        commands.entity(entity).insert(tier);
    }
}

// ─── System: apply LOD tier (visibility, render layers, step count) ───────────

pub fn apply_beam_lod(
    mut beam_cones: Query<
        (Entity, &BeamCone, &BeamLodTier, &mut Visibility, &MeshMaterial3d<BeamMaterial>),
        Without<BeamSprite>,
    >,
    beam_sprites: Query<(Entity, &BeamSprite, &mut Visibility), Without<BeamCone>>,
    mut beam_materials: ResMut<Assets<BeamMaterial>>,
    mut commands: Commands,
) {
    // Build lookup: FixtureId → sprite entity + visibility.
    let mut sprite_by_id = std::collections::HashMap::<FixtureId, Entity>::default();
    for (entity, sprite, _vis) in &beam_sprites {
        sprite_by_id.insert(sprite.id, entity);
    }

    for (entity, cone, tier, mut vis, mat_handle) in &mut beam_cones {
        // Toggle cone visibility.
        *vis = match tier {
            BeamLodTier::Tier0 => Visibility::Hidden,
            BeamLodTier::Tier1 | BeamLodTier::Tier2 => Visibility::Visible,
        };

        // Switch render layers.
        match tier {
            BeamLodTier::Tier1 => {
                commands.entity(entity).insert(RenderLayers::layer(1));
            }
            BeamLodTier::Tier0 | BeamLodTier::Tier2 => {
                commands.entity(entity).insert(RenderLayers::layer(0));
            }
        }

        // Set step count on material.
        if let Some(mat) = beam_materials.get_mut(mat_handle) {
            mat.step_count = match tier {
                BeamLodTier::Tier1 => 16,
                BeamLodTier::Tier2 => 32,
                BeamLodTier::Tier0 => 16,
            };
        }

        // Toggle matching sprite visibility.
        if let Some(&sprite_entity) = sprite_by_id.get(&cone.id) {
            let sprite_vis = match tier {
                BeamLodTier::Tier0 => Visibility::Visible,
                BeamLodTier::Tier1 | BeamLodTier::Tier2 => Visibility::Hidden,
            };
            commands.entity(sprite_entity).insert(sprite_vis);
        }
    }
}
