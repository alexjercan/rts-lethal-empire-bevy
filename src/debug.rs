use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use lethal_empire_bevy::terrain::{self, ChunkCoord, CHUNK_SIZE, CHUNK_TILE_SIZE};

use crate::GameStates;

#[derive(Debug, Default)]
pub(super) struct DebugModePlugin;

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

    let size = UVec2::splat(CHUNK_SIZE as u32);
    let tile_size = Vec2::splat(CHUNK_TILE_SIZE);
    let tile_coord = terrain::helpers::geometry::world_pos_to_global_coord(&point.xz(), &size, &tile_size);
    let tile_pos = terrain::helpers::geometry::global_coord_to_world_pos(&tile_coord, &size, &tile_size);

    gizmos.rect(
        tile_pos.extend(0.0).xzy(),
        Quat::from_rotation_x(FRAC_PI_2),
        tile_size,
        Color::WHITE,
    );
}

fn draw_chunks(mut gizmos: Gizmos, q_chunks: Query<&ChunkCoord>) {
    for coord in q_chunks.iter() {
        let chunk_size = CHUNK_SIZE as f32;
        let tile_size = CHUNK_TILE_SIZE;

        let position = (**coord).extend(0).xzy().as_vec3() * chunk_size * tile_size;
        gizmos.sphere(position, Quat::IDENTITY, 0.5, Color::RED);

        gizmos.rect(
            position,
            Quat::from_rotation_x(FRAC_PI_2),
            Vec2::splat(chunk_size * tile_size),
            Color::RED,
        );
    }
}
