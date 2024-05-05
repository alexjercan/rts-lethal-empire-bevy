use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use super::ToolMode;

pub fn setup(mut commands: Commands) {
    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
    ));
}

pub fn select_tool_mode(mut tool_mode: ResMut<ToolMode>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        *tool_mode = ToolMode::Select;
    } else if input.just_pressed(KeyCode::KeyB) {
        *tool_mode = ToolMode::Build;
    }
}
