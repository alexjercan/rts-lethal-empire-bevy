use bevy::prelude::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
    SeedableRng,
};

#[derive(Debug, Clone)]
pub(super) struct PoissonSampler {
    rng: rand::rngs::SmallRng,
}

impl PoissonSampler {
    pub(super) fn new(seed: u64) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub(super) fn sample(self, radius: f32, size: f32, k: u32) -> Vec<Vec2> {
        let cell_size = radius / 2.0_f32.sqrt();
        let grid_size = (size / cell_size).ceil() as usize;

        let mut grid = vec![None; grid_size * grid_size];
        let mut samples = Vec::new();
        let mut active = Vec::new();

        let mut uniform = Uniform::new(0.0, 1.0).sample_iter(self.rng);

        let initial = Vec2::new(
            uniform.next().unwrap() * size,
            uniform.next().unwrap() * size,
        );
        let initial_index = (initial.y / cell_size).floor() as usize * grid_size
            + (initial.x / cell_size).floor() as usize;
        grid[initial_index] = Some(samples.len());
        samples.push(initial);

        active.push(initial);
        while active.len() > 0 {
            let index = (uniform.next().unwrap() * active.len() as f32) as usize;
            let sample = active[index];

            let mut found = false;
            for _ in 0..k {
                let angle = uniform.next().unwrap() * 2.0 * std::f32::consts::PI;
                let direction = Vec2::new(angle.cos(), angle.sin());
                let distance = radius * (uniform.next().unwrap() + 1.0);
                let candidate = sample + direction * distance;

                if is_valid(
                    candidate, size, cell_size, radius, &grid, grid_size, &samples,
                ) {
                    let candidate_index = (candidate.y / cell_size).floor() as usize * grid_size
                        + (candidate.x / cell_size).floor() as usize;
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
}

fn is_valid(
    candidate: Vec2,
    size: f32,
    cell_size: f32,
    radius: f32,
    grid: &Vec<Option<usize>>,
    grid_size: usize,
    samples: &Vec<Vec2>,
) -> bool {
    if candidate.x >= 0.0 && candidate.x < size && candidate.y >= 0.0 && candidate.y < size {
        let coord = (candidate / cell_size).as_uvec2();
        let min_x = (coord.x as i32 - 2).max(0) as usize;
        let max_x = (coord.x as i32 + 2).min(grid_size as i32 - 1) as usize;
        let min_y = (coord.y as i32 - 2).max(0) as usize;
        let max_y = (coord.y as i32 + 2).min(grid_size as i32 - 1) as usize;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if let Some(index) = grid[y * grid_size + x] {
                    let other = Vec2::new(samples[index].x, samples[index].y);
                    if (candidate - other).length() < radius {
                        return false;
                    }
                }
            }
        }

        return true;
    }

    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poission_sampler_seedable() {
        let samples1 = PoissonSampler::new(0).sample(1.0, 32.0, 30);
        let samples2 = PoissonSampler::new(0).sample(1.0, 32.0, 30);
        assert_eq!(samples1, samples2);
    }
}
