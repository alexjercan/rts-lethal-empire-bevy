use bevy::prelude::*;

#[derive(Component)]
pub(super) struct Chunk;

#[derive(Component)]
pub(super) struct ChunkCoord(pub(super) IVec2);
