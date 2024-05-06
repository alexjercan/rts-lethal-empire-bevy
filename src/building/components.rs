use bevy::prelude::*;

use crate::terrain::ResourceKind;

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

#[derive(Component)]
pub struct BuildingHasWorker;

#[derive(Component, Default, PartialEq, Eq, Clone, Hash, Debug)]
pub enum BuildingKind {
    #[default]
    LumberMill,
    StoneQuarry,
}

impl Into<ResourceKind> for BuildingKind {
    fn into(self) -> ResourceKind {
        match self {
            BuildingKind::LumberMill => ResourceKind::Tree,
            BuildingKind::StoneQuarry => ResourceKind::Rock,
        }
    }
}
