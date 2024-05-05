use bevy::prelude::*;
use rand::{distributions::{Distribution, Uniform}, Rng};
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn seed_from_coord(seed: u64, coord: &IVec2) -> u64 {
    let mut hasher = DefaultHasher::new();

    seed.hash(&mut hasher);
    coord.x.hash(&mut hasher);
    coord.y.hash(&mut hasher);

    hasher.finish()
}

pub fn random_angle<R: Rng + ?Sized>(rng: &mut R) -> f32 {
    return Uniform::new(0.0, 1.0).sample(rng)
        * 2.0
        * std::f32::consts::PI;
}
