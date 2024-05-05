use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;

use crate::building::BuildingKind;

#[derive(AssetCollection, Resource, Clone)]
pub struct GameAssets {
    #[asset(
        paths(
            "textures/tiles/water.png",
            "textures/tiles/grass.png",
            "textures/tiles/barren.png",
        ),
        collection(typed)
    )]
    pub tiles: Vec<Handle<Image>>,
    #[asset(path = "models/lowpoly_tree/tree.gltf#Scene0")]
    pub tree: Handle<Scene>,
    #[asset(path = "models/lowpoly_tree/tree_snow.gltf#Scene0")]
    pub tree_dead: Handle<Scene>,
    #[asset(path = "models/lowpoly_stone/stone_tallA.glb#Scene0")]
    pub rock: Handle<Scene>,
    #[asset(
        paths(
            "models/lowpoly_buildings/lumber_mill.glb#Scene0",
            "models/lowpoly_buildings/stone_quarry.glb#Scene0",
        ),
        collection(mapped, typed)
    )]
    pub buildings: HashMap<BuildingKind, Handle<Scene>>,
}

impl MapKey for BuildingKind {
    fn from_asset_path(path: &bevy::asset::AssetPath) -> Self {
        let stem = path
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .expect("Path should be valid UTF-8")
            .to_string();
        match stem.as_str() {
            "lumber_mill" => BuildingKind::LumberMill,
            "stone_quarry" => BuildingKind::StoneQuarry,
            _ => panic!("Unknown building kind: {}", stem),
        }
    }
}
