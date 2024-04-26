use bevy::{ecs::system::CommandQueue, prelude::*, tasks::Task};
use noise::utils::NoiseMap;

#[derive(Component)]
pub(in crate::terrain) struct ChunkNoiseMap(pub(in crate::terrain) NoiseMap);

#[derive(Component)]
pub(super) struct ComputeNoiseMap(pub(super) Task<CommandQueue>);
