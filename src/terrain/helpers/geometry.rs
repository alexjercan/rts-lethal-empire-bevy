use bevy::prelude::*;

/// Calculates a [`Transform`] for a chunk that places it so that its center is at
/// `coord` in world space.
pub fn get_chunk_coord_transform(
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

/// Convert a tile index to a tile coordinate. Top left is (0, 0).
pub fn index_to_tile_coord(index: usize, size: &UVec2) -> UVec2 {
    let x = index % size.x as usize;
    let y = index / size.x as usize;

    return UVec2::new(x as u32, y as u32);
}

/// Convert a tile coordinate to a global tile coordinate, by taking into account the chunk.
pub fn tile_coord_to_global_coord(tile_coord: &UVec2, chunk_coord: &IVec2, size: &UVec2) -> IVec2 {
    let global_coord = *chunk_coord * size.as_ivec2() + tile_coord.as_ivec2() - size.as_ivec2() / 2;

    return global_coord;
}

/// Convert a tile coordinate to a world offset.
pub fn tile_coord_to_world_off(tile_coord: &UVec2, size: &UVec2, tile_size: &Vec2) -> Vec2 {
    let offset = (tile_coord.as_ivec2() - size.as_ivec2() / 2).as_vec2() * *tile_size + *tile_size / 2.0;

    return offset;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_tile_coord_transform_1() {
        let tile_coord = UVec2::new(1, 1);
        let size = UVec2::new(16, 16);
        let tile_size = Vec2::new(1.0, 1.0);

        let translation = get_tile_coord_translation(&tile_coord, &size, &tile_size, 0.0);

        assert_eq!(translation, Vec3::new(-6.5, 0.0, -6.5));
    }

    #[test]
    fn test_get_tile_coord_transform_2() {
        let tile_coord = UVec2::new(9, 9);
        let size = UVec2::new(16, 16);
        let tile_size = Vec2::new(2.0, 2.0);

        let translation = get_tile_coord_translation(&tile_coord, &size, &tile_size, 0.0);

        assert_eq!(translation, Vec3::new(3.0, 0.0, 3.0));
    }
}
