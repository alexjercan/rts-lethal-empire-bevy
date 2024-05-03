use bevy::prelude::*;

use crate::GameStates;

#[derive(Debug, Default)]
pub(super) struct DebugModePlugin;

impl Plugin for DebugModePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_cursor.run_if(in_state(GameStates::Playing)));
    }
}

fn draw_cursor(
    q_camera: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = q_camera.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y)) else {
        return;
    };
    let point = ray.get_point(distance);

    gizmos.circle(point + Vec3::Y * 0.01, Direction3d::Y, 0.2, Color::WHITE);
}
