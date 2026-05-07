pub mod beam;
pub mod fixture;
pub mod gobo;
pub mod scene;

use bevy::prelude::*;

pub struct StageLxRenderPlugin;

impl Plugin for StageLxRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, scene::setup_default_camera);
        // Phase 2: beam rendering, gobo projection, articulated fixture geometry
    }
}
