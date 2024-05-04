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

/// Calculates a [`Transform`] for a tile that places it so that its center is at
/// `coord` in world space.
pub fn get_tile_coord_transform(
    tile_coord: &UVec2,
    size: &UVec2,
    tile_size: &Vec2,
    y: f32,
) -> Transform {
    let translation = ((tile_coord.as_ivec2() - size.as_ivec2() / 2).as_vec2() * *tile_size + *tile_size / 2.0).extend(y).xzy();

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_tile_coord_transform_1() {
        let tile_coord = UVec2::new(1, 1);
        let size = UVec2::new(16, 16);
        let tile_size = Vec2::new(1.0, 1.0);

        let transform = get_tile_coord_transform(&tile_coord, &size, &tile_size, 0.0);

        assert_eq!(transform.translation, Vec3::new(-6.5, 0.0, -6.5));
    }

    #[test]
    fn test_get_tile_coord_transform_2() {
        let tile_coord = UVec2::new(9, 9);
        let size = UVec2::new(16, 16);
        let tile_size = Vec2::new(2.0, 2.0);

        let transform = get_tile_coord_transform(&tile_coord, &size, &tile_size, 0.0);

        assert_eq!(transform.translation, Vec3::new(3.0, 0.0, 3.0));
    }
}
