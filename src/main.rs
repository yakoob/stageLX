use bevy::prelude::*;
use stagelx_render::StageLxRenderPlugin;
use stagelx_ui::StageLxUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "stageLX".into(),
                resolution: (1600.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(StageLxRenderPlugin)
        .add_plugins(StageLxUiPlugin)
        .run();
}
