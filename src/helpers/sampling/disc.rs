use bevy::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    SeedableRng,
};

#[derive(Debug, Clone)]
pub struct PoissonDiscSampler {
    /// The seed to use for the disc sampler
    seed: u64,
    /// The radius of the discs that are sampled
    ///
    /// This represents the minimum distance between any two samples.
    radius: f32,
    /// The size of the area to sample.
    ///
    /// This is the size of the rectangle that the samples are placed in.
    size: Vec2,
    /// The number of candidates to generate for each sample.
    ///
    /// A higher value will result in more uniform sampling, but will also increase the time it
    /// takes to generate the samples.
    k: u32,
}

impl Default for PoissonDiscSampler {
    fn default() -> Self {
        Self {
            seed: 0,
            radius: 1.0,
            size: Vec2::splat(32.0),
            k: 30,
        }
    }
}

impl PoissonDiscSampler {
    pub fn new(seed: u64) -> Self {
        Self { seed, ..default() }
    }

    pub fn with_radius(self, radius: f32) -> Self {
        Self { radius, ..self }
    }

    pub fn with_size(self, size: Vec2) -> Self {
        Self { size, ..self }
    }

    pub fn with_k(self, k: u32) -> Self {
        Self { k, ..self }
    }

    pub fn sample(&self) -> Vec<Vec2> {
        let rng = StdRng::seed_from_u64(self.seed);
        let mut uniform = Uniform::new(0.0, 1.0).sample_iter(rng);

        let cell_size = self.radius / 2.0_f32.sqrt();
        let grid_size = (self.size / cell_size).ceil().as_uvec2();

        let mut grid = vec![None; (grid_size.x * grid_size.y) as usize];
        let mut samples = Vec::new();
        let mut active = Vec::new();

        let initial = Vec2::new(uniform.next().unwrap(), uniform.next().unwrap()) * self.size;
        let initial_coord = (initial / cell_size).as_uvec2();
        let initial_index = (initial_coord.y * grid_size.x + initial_coord.x) as usize;
        grid[initial_index] = Some(samples.len());
        samples.push(initial);

        active.push(initial);
        while active.len() > 0 {
            let index = (uniform.next().unwrap() * active.len() as f32) as usize;
            let sample = active[index];

            let mut found = false;
            for _ in 0..self.k {
                let angle = uniform.next().unwrap() * 2.0 * std::f32::consts::PI;
                let direction = Vec2::new(angle.cos(), angle.sin());
                let distance = self.radius * (uniform.next().unwrap() + 1.0);
                let candidate = sample + direction * distance;

                if self.is_valid(candidate, cell_size, &grid, grid_size, &samples) {
                    let candidate_coord = (candidate / cell_size).as_uvec2();
                    let candidate_index =
                        (candidate_coord.y * grid_size.x + candidate_coord.x) as usize;
                    grid[candidate_index] = Some(samples.len());
                    samples.push(candidate);
                    active.push(candidate);

                    found = true;
                    break;
                }
            }

            if !found {
                active.remove(index);
            }
        }

        return samples;
    }

    fn is_valid(
        &self,
        candidate: Vec2,
        cell_size: f32,
        grid: &Vec<Option<usize>>,
        grid_size: UVec2,
        samples: &Vec<Vec2>,
    ) -> bool {
        if Rect::from_corners(Vec2::ZERO, self.size).contains(candidate) {
            let coord = (candidate / cell_size).as_uvec2();
            let min_x = (coord.x as i32 - 2).max(0) as usize;
            let max_x = (coord.x as i32 + 2).min(grid_size.x as i32 - 1) as usize;
            let min_y = (coord.y as i32 - 2).max(0) as usize;
            let max_y = (coord.y as i32 + 2).min(grid_size.y as i32 - 1) as usize;

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if let Some(index) = grid[y * grid_size.x as usize + x] {
                        let other = Vec2::new(samples[index].x, samples[index].y);
                        if (candidate - other).length() < self.radius {
                            return false;
                        }
                    }
                }
            }

            return true;
        }

        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test::Bencher;

    #[bench]
    fn bench_poisson_disc_sampler(b: &mut Bencher) {
        let sampler = PoissonDiscSampler::new(0)
            .with_radius(1.0)
            .with_size(Vec2::splat(32.0))
            .with_k(30);

        b.iter(|| sampler.sample());
    }
}
