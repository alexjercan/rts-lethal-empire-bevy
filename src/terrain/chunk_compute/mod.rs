// Purpose: Add the `ChunkNoiseMap` component to the `Chunk` entity.
// Does not really matter how it is done, we just want a noise map to be associated with a chunk.

use bevy::prelude::*;
use noise::utils::NoiseMap;

use super::{
    components::{Chunk, ChunkCoord, ChunkNoiseMap},
    TerrainConfig,
};
use resources::ChunkComputeCPUConfig;
use systems::{handle_noise_map_tasks, spawn_noise_map_tasks};

impl From<NoiseMap> for ChunkNoiseMap {
    fn from(value: NoiseMap) -> Self {
        let size = value.size();
        let border_value = value.border_value();
        let map = value.into_iter().collect::<Vec<_>>();
        return ChunkNoiseMap {
            size: UVec2::new(size.0 as u32, size.1 as u32),
            border_value,
            map,
        };
    }
}

mod components;
mod resources;
mod systems;

#[derive(Debug, Default)]
pub(super) struct ChunkComputeCPUPlugin {
    config: ChunkComputeCPUConfig,
}

impl Plugin for ChunkComputeCPUPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.add_systems(Update, (spawn_noise_map_tasks, handle_noise_map_tasks));
    }
}
