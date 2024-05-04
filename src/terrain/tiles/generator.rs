use bevy::prelude::*;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Perlin,
};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum TileKind {
    #[default]
    Water,
    Grass,
    Barren,
}

impl TileKind {
    fn from_noise(noise: f64) -> Self {
        // TODO: multiple noise passes to generate patches of grass and barren land
        match noise {
            n if n < 0.1 => TileKind::Barren,
            n if n < 0.4 => TileKind::Grass,
            _ => TileKind::Water,
        }
    }
}

#[derive(Resource, Clone)]
pub struct TerrainGenerator {
    seed: u64,
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
            .map(|noise| TileKind::from_noise(noise))
            .collect()
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
}
