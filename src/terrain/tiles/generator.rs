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
    Barren
}

impl TileKind {
    fn from_noise(noise: f64) -> Self {
        // TODO: multiple noise passes to generate patches of grass and barren land
        match noise {
            n if n < 0.0 => TileKind::Water,
            n if n < 0.3 => TileKind::Grass,
            _ => TileKind::Barren,
        }
    }
}

#[derive(Resource, Deref)]
pub struct TerrainGenerator(pub Fbm<Perlin>);

impl Default for TerrainGenerator {
    fn default() -> Self {
        TerrainGenerator(
            Fbm::<Perlin>::new(0)
                .set_frequency(1.0)
                .set_persistence(0.5)
                .set_lacunarity(2.0)
                .set_octaves(14),
        )
    }
}

impl TerrainGenerator {
    pub fn generate(&self, coord: IVec2, size: UVec2) -> Vec<TileKind> {
        PlaneMapBuilder::new(self.0.clone())
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
        let generator = TerrainGenerator::default();

        b.iter(|| generator.generate(IVec2::ZERO, UVec2::splat(128)));
    }
}
