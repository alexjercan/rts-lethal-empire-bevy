use std::f32::consts::FRAC_PI_2;

use crate::{
    core::GameStates,
    helpers,
    terrain::{ChunkCoord, ChunkManager},
};
use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct DebugModePlugin;

impl Plugin for DebugModePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (draw_cursor, draw_chunks, draw_cursor_tile).run_if(in_state(GameStates::Playing)),
        );
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

fn draw_cursor_tile(
    q_camera: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
    chunk_manager: Res<ChunkManager>,
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

    let size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();
    let tile_pos = helpers::geometry::snap_to_tile(&point.xz(), &size, &tile_size);

    gizmos.rect(
        tile_pos.extend(0.0).xzy(),
        Quat::from_rotation_x(FRAC_PI_2),
        tile_size,
        Color::WHITE,
    );
}

fn draw_chunks(mut gizmos: Gizmos, q_chunks: Query<&ChunkCoord>, chunk_manager: Res<ChunkManager>) {
    for coord in q_chunks.iter() {
        let chunk_size = chunk_manager.size();
        let tile_size = chunk_manager.tile_size();

        let position = helpers::geometry::chunk_coord_to_world_pos(&coord, &chunk_size, &tile_size)
            .extend(0.0)
            .xzy();
        gizmos.sphere(position, Quat::IDENTITY, 0.5, Color::RED);

        gizmos.rect(
            position,
            Quat::from_rotation_x(FRAC_PI_2),
            chunk_size.as_vec2() * tile_size,
            Color::RED,
        );
    }
}
