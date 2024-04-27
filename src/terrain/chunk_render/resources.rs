use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Resource)]
pub(super) struct ChunkRenderCPUConfig {
    pub(super) forest_threshold_noise: f64,
}

impl Default for ChunkRenderCPUConfig {
    fn default() -> Self {
        ChunkRenderCPUConfig {
            forest_threshold_noise: 6144.0 / 16384.0,
        }
    }
}
