use bevy::prelude::*;
use noise::{
    core::worley::{distance_functions, ReturnType},
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Perlin, Worley,
};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum ResourceKind {
    #[default]
    None,
    Tree,
    Rock,
}

#[derive(Resource, Clone)]
pub struct ResourceGenerator {
    seed: u64,
}

impl ResourceGenerator {
    pub fn new(seed: u64) -> Self {
        ResourceGenerator { seed }
    }
}

impl ResourceGenerator {
    pub fn generate(&self, coord: IVec2, size: UVec2) -> Vec<ResourceKind> {
        let perlin = Fbm::<Perlin>::new(self.seed as u32)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(2.0)
            .set_octaves(14);

        let worley = Worley::new(self.seed as u32)
            .set_distance_function(distance_functions::euclidean)
            .set_return_type(ReturnType::Value)
            .set_frequency(1.0);

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
        let generator = ResourceGenerator::new(0);

        b.iter(|| generator.generate(IVec2::ZERO, UVec2::splat(128)));
    }
}
