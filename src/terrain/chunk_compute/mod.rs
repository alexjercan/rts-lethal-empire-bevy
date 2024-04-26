use bevy::prelude::*;

use super::{components::{Chunk, ChunkCoord}, TerrainConfig};
use systems::{handle_noise_map_tasks, spawn_noise_map_tasks};
use resources::ChunkComputeCPUConfig;

pub(super) use components::ChunkNoiseMap;

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
