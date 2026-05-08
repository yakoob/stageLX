pub mod io_panel;
pub mod library;
pub mod patch;
pub mod programmer;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

pub use stagelx_state::{
    DespawnFixtureEvent, FixtureLibraryRes, IoConfig, PatchEditState, PatchRes, Programmer,
    SpawnFixtureEvent,
};
pub use stagelx_render::VenueLoadState;

// ─── Plugin ───────────────────────────────────────────────────────────────────

use bevy_egui::egui;

pub struct StageLxUiPlugin;

impl Plugin for StageLxUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        app.init_resource::<Programmer>()
            .init_resource::<PatchRes>()
            .init_resource::<PatchEditState>()
            .init_resource::<FixtureLibraryRes>()
            .init_resource::<IoConfig>()
            .init_resource::<VenueLoadState>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_viewport_separators,
                    programmer::programmer_panel,
                    patch::patch_panel,
                    library::library_panel,
                    io_panel::io_panel,
                ),
            );
    }
}

// ─── Viewport separators ──────────────────────────────────────────────────────

fn draw_viewport_separators(mut ctx: bevy_egui::EguiContexts, windows: Query<&Window>) {
    let Ok(window) = windows.single() else { return };
    let w = window.width();
    let h = window.height();
    let split_x = w * 0.75;
    let split_y = h * 0.5;

    let egui_ctx = ctx.ctx_mut().expect("egui context");

    let stroke = egui::Stroke::new(1.5, egui::Color32::from_gray(70));
    let painter = egui_ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("_viewport_lines"),
    ));
    // Vertical separator between FOH and mini viewports
    painter.line_segment(
        [egui::pos2(split_x, 0.0), egui::pos2(split_x, h)],
        stroke,
    );
    // Horizontal separator between TOP and SIDE mini viewports
    painter.line_segment(
        [egui::pos2(split_x, split_y), egui::pos2(w, split_y)],
        stroke,
    );

    egui::Area::new(egui::Id::new("_label_top"))
        .fixed_pos(egui::pos2(split_x + 5.0, 5.0))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(egui_ctx, |ui| {
            ui.label(
                egui::RichText::new("TOP")
                    .size(11.0)
                    .color(egui::Color32::from_gray(130)),
            );
        });

    egui::Area::new(egui::Id::new("_label_side"))
        .fixed_pos(egui::pos2(split_x + 5.0, split_y + 5.0))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(egui_ctx, |ui| {
            ui.label(
                egui::RichText::new("SIDE")
                    .size(11.0)
                    .color(egui::Color32::from_gray(130)),
            );
        });
}
