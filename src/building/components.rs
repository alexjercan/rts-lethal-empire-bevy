use bevy::prelude::*;

#[derive(Component)]
pub struct BuildingTool;

#[derive(Component, Deref, DerefMut)]
pub(super) struct BuildingToolValid(pub bool);

#[derive(Component)]
pub(super) struct GhostBuilding;

#[derive(Component)]
pub(super) struct BuildingValidGhost;

#[derive(Component)]
pub struct Building;

#[derive(Component, Default, PartialEq, Eq, Clone, Hash, Debug)]
pub enum BuildingKind {
    #[default]
    LumberMill,
    StoneQuarry,
}
