use bevy::prelude::*;

/// Convert a chunk coordinate to a world position.
pub fn chunk_coord_to_world_pos(chunk_coord: &IVec2, size: &UVec2, tile_size: &Vec2) -> Vec2 {
    let chunk_size = size.as_vec2() * *tile_size;
    let world_pos = chunk_coord.as_vec2() * chunk_size;

    return world_pos;
}

/// Convert a world position to a chunk coordinate.
pub fn world_pos_to_chunk_coord(world_pos: &Vec2, size: &UVec2, tile_size: &Vec2) -> IVec2 {
    let chunk_size = size.as_vec2() * *tile_size;
    let world_pos = *world_pos + chunk_size / 2.0;
    let delta = IVec2::new((world_pos.x < 0.0) as i32, (world_pos.y < 0.0) as i32);
    let chunk_coord = (world_pos / chunk_size).as_ivec2() - delta;

    return chunk_coord;
}

pub fn world_area_to_chunk_coords(world_pos: &Vec2, tiles: u32, size: &UVec2, tile_size: &Vec2) -> Vec<IVec2> {
    let chunk_coord = world_pos_to_chunk_coord(world_pos, size, tile_size);
    let chunk_size = size.as_vec2();
    let chunk_radius = tiles as f32 / chunk_size.x.max(chunk_size.y);
    let chunk_radius = chunk_radius.ceil() as i32;
    let mut coords = Vec::new();

    for y in -chunk_radius..=chunk_radius {
        for x in -chunk_radius..=chunk_radius {
            let coord = chunk_coord + IVec2::new(x, y);

            coords.push(coord);
        }
    }

    return coords;
}

/// Convert a world position to a global tile coordinate.
pub fn world_pos_to_global_coord(world_pos: &Vec2, size: &UVec2, tile_size: &Vec2) -> IVec2 {
    let chunk_coord = world_pos_to_chunk_coord(world_pos, size, tile_size);
    let coord = world_pos_to_tile_coord(world_pos, size, tile_size);
    let global_coord = tile_coord_to_global_coord(&coord, &chunk_coord, size);

    return global_coord;
}

pub fn global_coord_to_world_pos(global_coord: &IVec2, size: &UVec2, tile_size: &Vec2) -> Vec2 {
    let chunk_coord = global_coord_to_chunk_coord(global_coord, size);
    let coord = global_coord_to_tile_coord(global_coord, size);
    let offset = tile_coord_to_world_off(&coord, size, tile_size);

    let world_pos = chunk_coord_to_world_pos(&chunk_coord, size, tile_size) + offset;

    return world_pos;
}

pub fn snap_to_tile(world_pos: &Vec2, size: &UVec2, tile_size: &Vec2) -> Vec2 {
    let tile_coord = world_pos_to_global_coord(world_pos, &size, &tile_size);
    let tile_pos = global_coord_to_world_pos(&tile_coord, &size, &tile_size);

    return tile_pos;
}

/// Convert a world position to a tile coordinate.
pub fn world_pos_to_tile_coord(world_pos: &Vec2, size: &UVec2, tile_size: &Vec2) -> UVec2 {
    let chunk_coord = world_pos_to_chunk_coord(world_pos, size, tile_size);
    let chunk_pos = chunk_coord_to_world_pos(&chunk_coord, size, tile_size);
    let chunk_start = chunk_pos - size.as_vec2() * *tile_size / 2.0;
    let offset = *world_pos - chunk_start;

    let coord = (offset / *tile_size).as_uvec2();

    return coord;
}

/// Convert a tile global coordinate to a tile coordinate.
pub fn global_coord_to_tile_coord(global_coord: &IVec2, size: &UVec2) -> UVec2 {
    let coord = *global_coord + (*size / 2).as_ivec2();

    return coord.rem_euclid(size.as_ivec2()).as_uvec2();
}

/// Convert a tile coordinate to a global tile coordinate, by taking into account the chunk.
pub fn tile_coord_to_global_coord(tile_coord: &UVec2, chunk_coord: &IVec2, size: &UVec2) -> IVec2 {
    let global_coord = *chunk_coord * size.as_ivec2() + tile_coord.as_ivec2() - size.as_ivec2() / 2;

    return global_coord;
}

pub fn global_coord_to_chunk_coord(global_coord: &IVec2, size: &UVec2) -> IVec2 {
    let tile_coord = global_coord_to_tile_coord(global_coord, size);
    let coord = (*global_coord + size.as_ivec2() / 2 - tile_coord.as_ivec2()) / size.as_ivec2();

    return coord;
}

/// Convert a tile coordinate to a world offset.
pub fn tile_coord_to_world_off(tile_coord: &UVec2, size: &UVec2, tile_size: &Vec2) -> Vec2 {
    let offset =
        (tile_coord.as_ivec2() - size.as_ivec2() / 2).as_vec2() * *tile_size + *tile_size / 2.0;

    return offset;
}

/// Convert a tile index to a tile coordinate. Top left is (0, 0).
pub fn index_to_tile_coord(index: usize, size: &UVec2) -> UVec2 {
    let x = index % size.x as usize;
    let y = index / size.x as usize;

    return UVec2::new(x as u32, y as u32);
}

pub fn tile_coord_to_index(tile_coord: &UVec2, size: &UVec2) -> usize {
    let index = tile_coord.y * size.x + tile_coord.x;

    return index as usize;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chunk_coord_to_world_pos() {
        let chunk_coord = IVec2::new(1, 1);
        let size = UVec2::new(2, 2);
        let tile_size = Vec2::new(4.0, 4.0);

        let pos = chunk_coord_to_world_pos(&chunk_coord, &size, &tile_size);

        assert_eq!(pos, Vec2::new(8.0, 8.0));

        let chunk_coord = IVec2::new(-1, 1);
        let size = UVec2::new(2, 2);
        let tile_size = Vec2::new(4.0, 4.0);

        let pos = chunk_coord_to_world_pos(&chunk_coord, &size, &tile_size);

        assert_eq!(pos, Vec2::new(-8.0, 8.0));
    }

    #[test]
    fn test_world_pos_to_chunk_coord() {
        let pos = Vec2::new(8.0, 8.0);
        let size = UVec2::new(2, 2);
        let tile_size = Vec2::new(4.0, 4.0);

        let coord = world_pos_to_chunk_coord(&pos, &size, &tile_size);

        assert_eq!(coord, IVec2::new(1, 1));

        let pos = Vec2::new(-8.0, 8.0);
        let size = UVec2::new(2, 2);
        let tile_size = Vec2::new(4.0, 4.0);

        let coord = world_pos_to_chunk_coord(&pos, &size, &tile_size);

        assert_eq!(coord, IVec2::new(-1, 1));
    }

    #[test]
    fn test_world_pos_to_global_coord() {
        let pos = Vec2::new(8.0, 8.0);
        let size = UVec2::new(2, 2);
        let tile_size = Vec2::new(4.0, 4.0);

        let coord = world_pos_to_global_coord(&pos, &size, &tile_size);

        assert_eq!(coord, IVec2::new(2, 2));

        let pos = Vec2::new(-8.0, 8.0);
        let size = UVec2::new(2, 2);
        let tile_size = Vec2::new(4.0, 4.0);

        let coord = world_pos_to_global_coord(&pos, &size, &tile_size);

        assert_eq!(coord, IVec2::new(-2, 2));
    }

    #[test]
    fn test_world_pos_to_tile_coord() {
        let pos = Vec2::new(8.0, 8.0);
        let size = UVec2::new(2, 2);
        let tile_size = Vec2::new(4.0, 4.0);

        let coord = world_pos_to_tile_coord(&pos, &size, &tile_size);

        assert_eq!(coord, UVec2::new(1, 1));

        let pos = Vec2::new(-8.0, 8.0);
        let size = UVec2::new(2, 2);
        let tile_size = Vec2::new(4.0, 4.0);

        let coord = world_pos_to_tile_coord(&pos, &size, &tile_size);

        assert_eq!(coord, UVec2::new(1, 1));
    }

    #[test]
    fn test_index_to_tile_coord() {
        let size = UVec2::new(2, 2);
        let index = 3;

        let coord = index_to_tile_coord(index, &size);

        assert_eq!(coord, UVec2::new(1, 1));
    }

    #[test]
    fn test_global_coord_to_tile_coord() {
        let size = UVec2::new(2, 2);
        let global_coord = IVec2::new(8, 8);

        let coord = global_coord_to_tile_coord(&global_coord, &size);

        assert_eq!(coord, UVec2::new(1, 1));

        let size = UVec2::new(2, 2);
        let global_coord = IVec2::new(-8, 8);

        let coord = global_coord_to_tile_coord(&global_coord, &size);

        assert_eq!(coord, UVec2::new(1, 1));
    }

    #[test]
    fn test_tile_coord_to_world_off() {
        let size = UVec2::new(16, 16);
        let tile_coord = UVec2::new(12, 12);
        let tile_size = Vec2::new(32.0, 32.0);

        let offset = tile_coord_to_world_off(&tile_coord, &size, &tile_size);

        assert_eq!(offset, Vec2::new(144.0, 144.0));
    }
}
