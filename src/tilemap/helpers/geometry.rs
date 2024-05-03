use bevy::prelude::*;

/// Calculates a [`Transform`] for a tilemap that places it so that its center is at
/// `coord` in world space.
pub fn get_tilemap_coord_transform(
    coord: &IVec2,
    size: &UVec2,
    tile_size: &Vec2,
    y: f32,
) -> Transform {
    let translation = chunk_coord_to_world_pos(coord, size, tile_size)
        .extend(y)
        .xzy();

    return Transform::from_translation(translation);
}

/// Calculates a [`Transform`] for a tile that places it so that its center is at
/// `coord` in world space.
pub fn get_tile_coord_transform(
    coord: &IVec2,
    size: &UVec2,
    tile_size: &Vec2,
    y: f32,
) -> Transform {
    let translation = tile_coord_to_world_offset(coord, size, tile_size)
        .extend(y)
        .xzy();

    return Transform::from_translation(translation);
}

pub fn tile_coord_to_world_offset(coord: &IVec2, size: &UVec2, tile_size: &Vec2) -> Vec2 {
    let offset =
        coord.as_vec2() * *tile_size - size.as_vec2() * *tile_size / 2.0 + *tile_size / 2.0;

    return offset;
}

pub fn chunk_coord_to_world_pos(chunk_coord: &IVec2, size: &UVec2, tile_size: &Vec2) -> Vec2 {
    let chunk_size = size.as_vec2() * *tile_size;
    let world_pos = chunk_coord.as_vec2() * chunk_size;

    return world_pos;
}

pub fn world_pos_to_chunk_coord(world_pos: &Vec2, size: &UVec2, tile_size: &Vec2) -> IVec2 {
    let chunk_size = size.as_vec2() * *tile_size;
    let chunk_coord = (*world_pos / chunk_size).floor().as_ivec2();

    return chunk_coord;
}
