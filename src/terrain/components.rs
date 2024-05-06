use super::{ResourceKind, TileKind};
use bevy::{ecs::system::CommandQueue, prelude::*, tasks::Task};

#[derive(Component, Deref)]
pub struct ChunkCoord(pub IVec2);

#[derive(Component, Deref)]
pub struct TileCoord(pub UVec2);

#[derive(Component, Deref)]
pub struct TileMapping(pub Vec<TileKind>);

#[derive(Component, Deref)]
pub struct ResourceMapping(pub Vec<ResourceKind>);

#[derive(Component)]
pub(super) struct ChunkHandledTiles;

#[derive(Component)]
pub(super) struct ChunkHandledResources;

#[derive(Component)]
pub(super) struct ComputeTileMapping(pub Task<CommandQueue>);

#[derive(Component)]
pub(super) struct ComputeResourceMapping(pub Task<CommandQueue>);
