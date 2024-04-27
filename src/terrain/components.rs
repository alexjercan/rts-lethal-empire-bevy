use bevy::prelude::*;

#[derive(Component)]
pub(super) struct Chunk;

#[derive(Component)]
pub(super) struct ChunkCoord(pub(super) IVec2);

pub(super) struct NoiseMapData {
    pub(super) size: UVec2,
    pub(super) border_value: f64,
    pub(super) map: Vec<f64>,
}

pub(super) struct TreeData {
    pub(super) position: Vec2,
    pub(super) noise: f64,
}

pub(super) struct RockData {
    pub(super) position: Vec2,
    pub(super) noise: f64,
}

#[derive(Component)]
pub(super) struct ChunkData {
    pub(super) terrain: NoiseMapData,
    pub(super) forest: Vec<TreeData>,
    pub(super) rocks: Vec<RockData>,
}
