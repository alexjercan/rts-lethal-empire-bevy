use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Resource)]
pub(super) struct TerrainConfig {
    pub(super) seed: u64,
    pub(super) chunk_size: f32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        TerrainConfig {
            seed: 0,
            chunk_size: 32.0,
        }
    }
}
