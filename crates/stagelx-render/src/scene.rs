use bevy::{prelude::*, camera::{ScalingMode, Viewport, visibility::RenderLayers}, light::GlobalAmbientLight, window::WindowResized};
use crate::camera::FohCameraController;

#[derive(Component)]
pub struct FohCamera;

#[derive(Component)]
pub struct TopCamera;

#[derive(Component)]
pub struct SideCamera;

// Logical-pixel sizes of the egui panels surrounding the viewport area.
// These MUST match the egui panel sizes in stagelx-ui (`ui_root_system`):
//   left_rail  .exact_width(300)   right_rail .exact_width(420)
//   top_bar    .exact_height(36)   status_bar .exact_height(22)
//   bottom_strip .exact_height(248)
// A mismatch lets the 3-D viewports slide under an egui panel.
const LEFT_PANEL: f32 = 300.0;
const RIGHT_PANEL: f32 = 420.0;
const TOP_BAR: f32 = 36.0;
const STATUS_BAR: f32 = 22.0;
const BOTTOM_STRIP: f32 = 248.0;

fn compute_viewports(pw: u32, ph: u32, sf: f32) -> (Viewport, Viewport, Viewport) {
    let left  = (LEFT_PANEL   * sf).round() as u32;
    let right = (RIGHT_PANEL  * sf).round() as u32;
    let top   = (TOP_BAR      * sf).round() as u32;
    let bot   = (STATUS_BAR   * sf).round() as u32;
    let strip = (BOTTOM_STRIP * sf).round() as u32;

    let vp_w = pw.saturating_sub(left + right);
    let vp_h = ph.saturating_sub(top + bot + strip);

    let foh_w  = (vp_w as f32 * 0.75) as u32;
    let mini_w = vp_w.saturating_sub(foh_w);
    let mini_h = vp_h / 2;

    (
        Viewport {
            physical_position: UVec2::new(left, top),
            physical_size: UVec2::new(foh_w, vp_h),
            depth: 0.0..1.0,
        },
        Viewport {
            physical_position: UVec2::new(left + foh_w, top),
            physical_size: UVec2::new(mini_w, mini_h),
            depth: 0.0..1.0,
        },
        Viewport {
            physical_position: UVec2::new(left + foh_w, top + mini_h),
            physical_size: UVec2::new(mini_w, vp_h.saturating_sub(mini_h)),
            depth: 0.0..1.0,
        },
    )
}

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    windows: Query<&Window>,
) {
    let (w, h, sf) = windows
        .single()
        .map(|win| (win.physical_width(), win.physical_height(), win.scale_factor()))
        .unwrap_or((1440, 900, 2.0));

    let (foh_vp, top_vp, side_vp) = compute_viewports(w, h, sf);

    // Stage floor
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40.0, 24.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.08, 0.08),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.01, 0.0),
    ));

    // Truss bar (visual only — fixtures mount to this)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(18.0, 0.12, 0.12))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.4, 0.15),
            metallic: 0.2,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 6.0, 0.0),
    ));

    // FOH perspective camera — main view (left 75%)
    // Layer 0 = scene geometry; layer 4 = FOH-only beam visuals (Tier-2 full-res
    // cones + Tier-0 billboards), kept off the ortho cameras' layers.
    commands.spawn((
        FohCamera,
        Camera3d::default(),
        Camera {
            viewport: Some(foh_vp),
            ..default()
        },
        RenderLayers::layer(0) | RenderLayers::layer(4),
        FohCameraController::default(),
    ));

    // Top-down orthographic camera — right 25%, upper half
    commands.spawn((
        TopCamera,
        Camera3d::default(),
        Camera {
            viewport: Some(top_vp),
            order: 1,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 30.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        // Layer 0 = geometry; layer 3 = ray-marched beam cones (all tiers).
        RenderLayers::layer(0) | RenderLayers::layer(3),
        // Positioned straight above; Z is "up" in this 2-D view (stage depth)
        Transform::from_xyz(0.0, 40.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    // Side orthographic camera (stage right → center) — right 25%, lower half
    commands.spawn((
        SideCamera,
        Camera3d::default(),
        Camera {
            viewport: Some(side_vp),
            order: 2,
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 20.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        // Layer 0 = geometry; layer 3 = ray-marched beam cones (all tiers).
        RenderLayers::layer(0) | RenderLayers::layer(3),
        Transform::from_xyz(40.0, 3.0, 0.0).looking_at(Vec3::new(0.0, 3.0, 0.0), Vec3::Y),
    ));

    // Ambient lift so the stage geometry and fixtures read off black.
    //
    // NOTE: In Bevy 0.18 `AmbientLight` is a `#[require(Camera)]` component —
    // spawning it on its own entity does nothing (it just creates an orphan
    // camera). The scene-wide ambient is the `GlobalAmbientLight` resource.
    // The additive beam cones use an unlit custom material, so raising ambient
    // brightens the geometry without washing out the beams.
    // The camera uses Bevy's default (Blender, EV100 9.7) exposure tuned for
    // daylight, so stage-scale light needs to be in the thousands of lux to
    // register. Floor albedo is low (0.08) to keep the stage dark; fixtures use
    // higher albedo so they read several times brighter than the floor.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.7, 0.7, 0.8),
        brightness: 1800.0,
        affects_lightmapped_meshes: true,
    });

    // Key light from front-above. Metallic fixture bodies have almost no diffuse
    // response, so ambient alone leaves them flat — a directional light gives
    // them specular shading and reads them as solid 3-D objects.
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.95, 0.95, 1.0),
            illuminance: 5000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(6.0, 12.0, 10.0).looking_at(Vec3::new(0.0, 4.0, 0.0), Vec3::Y),
    ));
}

#[allow(clippy::type_complexity)]
pub fn update_viewports_on_resize(
    mut resize_events: MessageReader<WindowResized>,
    windows: Query<&Window>,
    mut cameras: Query<(
        &mut Camera,
        Option<&FohCamera>,
        Option<&TopCamera>,
        Option<&SideCamera>,
    )>,
) {
    if resize_events.is_empty() {
        return;
    }
    resize_events.clear();

    let Ok(window) = windows.single() else {
        return;
    };
    let w = window.physical_width();
    let h = window.physical_height();
    if w == 0 || h == 0 {
        return;
    }
    let sf = window.scale_factor();
    let (foh_vp, top_vp, side_vp) = compute_viewports(w, h, sf);

    for (mut cam, foh, top, side) in &mut cameras {
        cam.viewport = Some(if foh.is_some() {
            foh_vp.clone()
        } else if top.is_some() {
            top_vp.clone()
        } else if side.is_some() {
            side_vp.clone()
        } else {
            continue;
        });
    }
}
