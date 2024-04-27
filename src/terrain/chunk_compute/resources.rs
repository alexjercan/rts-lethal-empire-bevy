use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Resource)]
pub(super) struct ChunkComputeCPUConfig {
    pub(super) terrain_frequency: f64,
    pub(super) forest_frequency: f64,
    pub(super) rock_frequency: f64,
    pub(super) persistence: f64,
    pub(super) lacunarity: f64,
    pub(super) octaves: u32,
    pub(super) scale: f32,
    pub(super) bounds_interval: f64,
    pub(super) forest_min_noise: f64,
    pub(super) forest_max_noise: f64,
    pub(super) forest_tree_radius: f32,
    pub(super) forest_discard_threshold: f64,
    pub(super) forest_k: u32,
    pub(super) rock_min_noise: f64,
    pub(super) rock_max_noise: f64,
    pub(super) rock_radius: f32,
    pub(super) rock_discard_threshold: f64,
    pub(super) rock_k: u32,
}

impl Default for ChunkComputeCPUConfig {
    fn default() -> Self {
        ChunkComputeCPUConfig {
            terrain_frequency: 1.0,
            forest_frequency: 10.0,
            rock_frequency: 1.0,
            persistence: 0.5,
            lacunarity: 2.0,
            octaves: 14,
            scale: 8.0,
            bounds_interval: 0.125,
            forest_min_noise: 4096.0 / 16384.0,
            forest_max_noise: 1.0,
            forest_tree_radius: 2.0,
            forest_discard_threshold: -0.2,
            forest_k: 30,
            rock_min_noise: 1024.0 / 16384.0,
            rock_max_noise: 4096.0 / 16384.0,
            rock_radius: 4.0,
            rock_discard_threshold: -0.75,
            rock_k: 30,
        }
    }
}
