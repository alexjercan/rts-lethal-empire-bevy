use bevy::prelude::*;

#[derive(Component)]
pub(super) struct Chunk;

#[derive(Component)]
pub(super) struct ChunkCoord(pub(super) IVec2);

#[derive(Component)]
pub(super) struct ChunkNoiseMap {
    pub(super) size: UVec2,
    pub(super) border_value: f64,
    pub(super) map: Vec<f64>,
}
