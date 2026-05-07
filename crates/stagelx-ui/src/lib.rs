pub mod library;
pub mod patch;
pub mod programmer;

use bevy::prelude::*;

pub struct StageLxUiPlugin;

impl Plugin for StageLxUiPlugin {
    fn build(&self, _app: &mut App) {
        // Phase 1: add EguiPlugin and UI systems once bevy_egui version is confirmed.
        // Uncomment in stagelx-ui/Cargo.toml and here when ready:
        //   app.add_plugins(bevy_egui::EguiPlugin);
        //   app.add_systems(Update, patch::patch_panel);
        //   app.add_systems(Update, programmer::programmer_panel);
        //   app.add_systems(Update, library::library_panel);
    }
}
