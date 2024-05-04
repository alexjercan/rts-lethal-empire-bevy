use bevy::prelude::*;
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn seed_from_coord(seed: u64, coord: &IVec2) -> u64 {
    let mut hasher = DefaultHasher::new();

    seed.hash(&mut hasher);
    coord.x.hash(&mut hasher);
    coord.y.hash(&mut hasher);

    hasher.finish()
}
