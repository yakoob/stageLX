use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::core_3d::graph::Core3d;
use bevy::log::LogPlugin;
use bevy::render::camera::CameraRenderGraph;
use bevy_egui::{EguiContext, EguiGlobalSettings, EguiPlugin, PrimaryEguiContext};
use stagelx_dmx::cue_to_dmx;
use stagelx_io::IoPlugin;
use stagelx_render::StageLxRenderPlugin;
use stagelx_show::{
    load_cue_stack_on_startup,
    on_back_cue, on_delete_cue, on_go_cue, on_record_cue,
};
use stagelx_ui::StageLxUiPlugin;

fn setup_ui_camera(mut commands: Commands) {
    // Spawn CameraRenderGraph first to avoid the Camera on_add hook warning.
    let cam = commands.spawn((
        CameraRenderGraph::new(Core3d),
        Camera3d::default(),
    )).id();
    commands.entity(cam).insert((
        Camera {
            order: 100,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        // Layer 31 has no 3D entities — prevents the egui camera from
        // re-rendering the scene over the full window.
        RenderLayers::layer(31),
        EguiContext::default(),
        PrimaryEguiContext,
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "warn,bevy_render::camera=error".to_string(),
            ..default()
        }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "stageLX — Phase 6".into(),
                resolution: (1440_u32, 900_u32).into(),
                resize_constraints: WindowResizeConstraints {
                    min_width: 720.0,
                    min_height: 480.0,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        })
        .add_systems(PreStartup, setup_ui_camera)
        .add_systems(Startup, (load_cue_stack_on_startup, print_controls).chain())
        .add_plugins(StageLxUiPlugin)
        .add_plugins(StageLxRenderPlugin)
        .add_plugins(IoPlugin)
        // Cue event handlers (observers)
        .add_observer(on_record_cue)
        .add_observer(on_go_cue)
        .add_observer(on_back_cue)
        .add_observer(on_delete_cue)
        // Cue playback (priority 150) writes before engine merge.
        .add_systems(FixedUpdate, cue_to_dmx)
        .run();
}

fn print_controls() {
    info!("stageLX controls:");
    info!("  Arrow keys  — Pan / Tilt");
    info!("  + / -       — Dimmer up / down");
    info!("  W           — White");
    info!("  X           — Red");
    info!("  C           — Cyan/blue");
    info!("  R/G/B       — Nudge red/green/blue channel up");
    info!("  Enter       — Cue GO");
    info!("  Shift+Enter — Cue BACK");
    info!("  Right-drag  — Orbit FOH camera");
    info!("  Middle-drag — Pan FOH camera");
    info!("  Scroll      — Zoom FOH camera");
}
