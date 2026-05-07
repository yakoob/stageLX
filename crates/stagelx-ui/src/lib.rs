pub mod library;
pub mod patch;
pub mod programmer;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct StageLxUiPlugin;

impl Plugin for StageLxUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }
        // Phase 1 UI systems — wired in as panels are implemented:
        // app.add_systems(Update, patch::patch_panel);
        // app.add_systems(Update, programmer::programmer_panel);
        // app.add_systems(Update, library::library_panel);
    }
}
