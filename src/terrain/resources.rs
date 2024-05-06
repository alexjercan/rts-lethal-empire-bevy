use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use noise::{
    core::worley::{distance_functions, ReturnType},
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Perlin, Worley,
};

use super::{ResourceKind, TileKind, CHUNK_SIZE, CHUNK_TILE_SIZE};

#[derive(Resource, Deref)]
pub(super) struct TerrainSeed(pub u64);

#[derive(Resource, Clone)]
pub(super) struct TerrainGenerator {
    seed: u64,
}

#[derive(Resource, Clone)]
pub(super) struct ResourceGenerator {
    seed: u64,
}

#[derive(Debug, Resource)]
pub struct ChunkManager {
    size: UVec2,
    tile_size: Vec2,
    chunks: HashMap<IVec2, Entity>,
    loaded: HashSet<IVec2>,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            size: UVec2::splat(CHUNK_SIZE as u32),
            tile_size: Vec2::splat(CHUNK_TILE_SIZE),
            chunks: HashMap::new(),
            loaded: HashSet::new(),
        }
    }
}

impl ChunkManager {
    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn tile_size(&self) -> Vec2 {
        self.tile_size
    }

    pub fn load(&mut self, coord: IVec2) {
        self.loaded.insert(coord);
    }

    pub fn unload(&mut self, coord: IVec2) {
        self.loaded.remove(&coord);
    }

    pub fn out_range(&self, coord: &IVec2, radius: i32) -> Vec<IVec2> {
        self.loaded
            .iter()
            .filter(|c| (*coord - **c).abs().max_element() > radius)
            .cloned()
            .collect()
    }

    pub fn insert(&mut self, coord: IVec2, entity: Entity) {
        self.chunks.insert(coord, entity);
    }

    pub fn get(&self, coord: &IVec2) -> Option<&Entity> {
        self.chunks.get(coord)
    }

    pub fn contains(&self, coord: &IVec2) -> bool {
        self.chunks.contains_key(coord)
    }

    pub fn loaded(&self, coord: &IVec2) -> bool {
        self.loaded.contains(coord)
    }
}

impl TerrainGenerator {
    pub fn new(seed: u64) -> Self {
        TerrainGenerator { seed }
    }
}

impl TerrainGenerator {
    pub fn generate(&self, coord: IVec2, size: UVec2) -> Vec<TileKind> {
        let perlin = Fbm::<Perlin>::new(self.seed as u32)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0)
            .set_octaves(14);

        PlaneMapBuilder::new(perlin)
            .set_size(size.x as usize, size.y as usize)
            .set_x_bounds((coord.x as f64) * 1.0 - 0.5, (coord.x as f64) * 1.0 + 0.5)
            .set_y_bounds((coord.y as f64) * 1.0 - 0.5, (coord.y as f64) * 1.0 + 0.5)
            .build()
            .into_iter()
            .map(|noise| match noise {
                n if n < 0.1 => TileKind::Barren,
                n if n < 0.4 => TileKind::Grass,
                _ => TileKind::Water,
            })
            .collect()
    }
}

impl ResourceGenerator {
    pub fn new(seed: u64) -> Self {
        ResourceGenerator { seed }
    }
}

impl ResourceGenerator {
    pub fn generate(&self, coord: IVec2, size: UVec2) -> Vec<ResourceKind> {
        let perlin = Fbm::<Perlin>::new(self.seed as u32)
            .set_frequency(2.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0)
            .set_octaves(14);

        let worley = Worley::new(self.seed as u32)
            .set_distance_function(distance_functions::euclidean)
            .set_return_type(ReturnType::Value)
            .set_frequency(2.0);

        return PlaneMapBuilder::new(perlin)
            .set_size(size.x as usize, size.y as usize)
            .set_x_bounds((coord.x as f64) * 1.0 - 0.5, (coord.x as f64) * 1.0 + 0.5)
            .set_y_bounds((coord.y as f64) * 1.0 - 0.5, (coord.y as f64) * 1.0 + 0.5)
            .build()
            .into_iter()
            .zip(
                PlaneMapBuilder::new(worley)
                    .set_size(size.x as usize, size.y as usize)
                    .set_x_bounds((coord.x as f64) * 1.0 - 0.5, (coord.x as f64) * 1.0 + 0.5)
                    .set_y_bounds((coord.y as f64) * 1.0 - 0.5, (coord.y as f64) * 1.0 + 0.5)
                    .build()
                    .into_iter(),
            )
            .map(|(noise, worley)| {
                if worley < 0.0 || noise < 0.3 {
                    ResourceKind::None
                } else if worley < 0.5 {
                    ResourceKind::Rock
                } else {
                    ResourceKind::Tree
                }
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test::Bencher;

    #[bench]
    fn bench_terrain_generator(b: &mut Bencher) {
        let generator = TerrainGenerator::new(0);

        b.iter(|| generator.generate(IVec2::ZERO, UVec2::splat(128)));
    }

    #[bench]
    fn bench_resource_generator(b: &mut Bencher) {
        let generator = ResourceGenerator::new(0);

        b.iter(|| generator.generate(IVec2::ZERO, UVec2::splat(128)));
    }
}
