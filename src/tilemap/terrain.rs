use bevy::prelude::*;
use noise::{utils::{NoiseMapBuilder, PlaneMapBuilder}, Fbm, MultiFractal, Perlin};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum TerrainKind {
    #[default]
    Water,
    Grass,
    Forest,
    Rock,
}

impl TerrainKind {
    fn from_noise(noise: f64) -> Self {
        match noise {
            n if n < 0.0 => TerrainKind::Water,
            n if n < 0.2 => TerrainKind::Grass,
            n if n < 0.4 => TerrainKind::Forest,
            _ => TerrainKind::Rock,
        }
    }
}

#[derive(Resource, Deref)]
pub struct TerrainGenerator(Fbm<Perlin>);

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
    pub fn generate(&self, coord: IVec2, size: UVec2) -> Vec<TerrainKind> {
        PlaneMapBuilder::new(self.0.clone())
            .set_size(size.x as usize, size.y as usize)
            .set_x_bounds((coord.x as f64) * 1.0 - 0.5, (coord.x as f64) * 1.0 + 0.5)
            .set_y_bounds((coord.y as f64) * 1.0 - 0.5, (coord.y as f64) * 1.0 + 0.5)
            .build()
            .into_iter()
            .map(|noise| TerrainKind::from_noise(noise))
            .collect()
    }
}