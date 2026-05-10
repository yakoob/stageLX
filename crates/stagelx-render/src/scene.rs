use bevy::{prelude::*, camera::{ScalingMode, Viewport, visibility::RenderLayers}, window::WindowResized};
use crate::camera::FohCameraController;

#[derive(Component)]
pub struct FohCamera;

#[derive(Component)]
pub struct TopCamera;

#[derive(Component)]
pub struct SideCamera;

// Logical-pixel sizes of the egui panels surrounding the viewport area.
const LEFT_PANEL: f32 = 300.0;
const RIGHT_PANEL: f32 = 320.0;
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
        .map(|win| (win.physical_width(), win.physical_height(), win.scale_factor() as f32))
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
            base_color: Color::srgb(0.4, 0.35, 0.1),
            metallic: 0.6,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_xyz(0.0, 6.0, 0.0),
    ));

    // FOH perspective camera — main view (left 75%)
    commands.spawn((
        FohCamera,
        Camera3d::default(),
        Camera {
            viewport: Some(foh_vp),
            ..default()
        },
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
        RenderLayers::layer(0) | RenderLayers::layer(2),
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
        RenderLayers::layer(0) | RenderLayers::layer(2),
        Transform::from_xyz(40.0, 3.0, 0.0).looking_at(Vec3::new(0.0, 3.0, 0.0), Vec3::Y),
    ));

    // Dim ambient so beam lights are visible (Bevy 0.18: component, not Resource)
    commands.spawn(AmbientLight {
        color: Color::srgb(0.04, 0.04, 0.07),
        brightness: 80.0,
        affects_lightmapped_meshes: true,
    });
}

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
    let sf = window.scale_factor() as f32;
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
