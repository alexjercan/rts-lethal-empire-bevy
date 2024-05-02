use bevy::prelude::*;

/// Calculates a [`Transform`] for a tilemap that places it so that its center is at
/// `coord` in world space.
pub fn get_tilemap_coord_transform(coord: &IVec2, size: &UVec2, tile_size: &Vec2, y: f32) -> Transform {
    let chunk_size = size.as_vec2() * *tile_size;
    let translation = (coord.as_vec2() * chunk_size).extend(y).xzy();

    return Transform::from_translation(translation);
}

pub fn world_pos_to_chunk_coord(world_pos: &Vec2, size: &UVec2, tile_size: &Vec2) -> IVec2 {
    let chunk_size = size.as_vec2() * *tile_size;
    let chunk_coord = (*world_pos / chunk_size).floor().as_ivec2();

    return chunk_coord;
}
