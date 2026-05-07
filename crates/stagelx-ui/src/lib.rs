pub mod io_panel;
pub mod library;
pub mod patch;
pub mod programmer;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

pub use stagelx_state::{FixtureLibraryRes, IoConfig, PatchRes, Programmer};

// ─── Plugin ───────────────────────────────────────────────────────────────────

pub struct StageLxUiPlugin;

impl Plugin for StageLxUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        app.init_resource::<Programmer>()
            .init_resource::<PatchRes>()
            .init_resource::<FixtureLibraryRes>()
            .init_resource::<IoConfig>()
            .add_systems(
                EguiPrimaryContextPass,
                (
                    programmer::programmer_panel,
                    patch::patch_panel,
                    library::library_panel,
                    io_panel::io_panel,
                ),
            );
    }
}
