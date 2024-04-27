use bevy::{ecs::system::CommandQueue, prelude::*, tasks::Task};

#[derive(Component)]
pub(super) struct ComputeNoiseMap(pub(super) Task<CommandQueue>);
