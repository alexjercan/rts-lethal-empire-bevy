use bevy::prelude::*;

/// Calculates a [`Transform`] for a tilemap that places it so that its center is at
/// `(0.0, 0.0, 0.0)` in world space.
pub fn get_tilemap_center_transform(size: &UVec2, tile_size: &Vec2, y: f32) -> Transform {
    let low = Vec2::ZERO;
    let high = size.as_vec2() * *tile_size;

    let diff = high - low;

    return Transform::from_xyz(-diff.x / 2., y, -diff.y / 2.);
}
