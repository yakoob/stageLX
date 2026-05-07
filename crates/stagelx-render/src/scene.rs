use bevy::prelude::*;

pub fn setup_default_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light for the venue floor
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.05, 0.05, 0.1),
        brightness: 100.0,
    });
}
