use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

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
}
