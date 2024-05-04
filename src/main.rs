use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use lethal_empire_bevy::{
    assets::GameAssets, states::GameStates, terrain::TerrainPlugin,
};

#[cfg(feature = "debug")]
mod debug;

#[cfg(feature = "debug")]
use debug::DebugModePlugin;

// Features that I want in my game:
//
// # Version 0.1
// - [x] Tile based map
//   - [x] create a perlin noise generator that can give us noise values for a given area (chunk)
//   - [x] implement a mapper from noise values to tile types
//   - [x] create a custom atlas with all the tile types and textures
//   - [x] write a shader that takes in the tile types (RGB) and the atlas and renders the map
// - [x] Resources
//   - [x] implement a system that randomly generates resources on the map
//   - [x] implement a new tile type for trees and rocks
//   - [x] add models for trees and rocks and spawn them in the world
//   - [x] think about a better way than just random for V2
// - [ ] Buildings
//   - [ ] extremely simple buildings that can be placed on the map and give us resources over time
// - [ ] Main Goal
//   - [ ] need to pay quota of resources to the Empire over time
//   - [ ] UI with the timer and quota needed and also how much we have
//   - "TIME LEFT: 10:00" "QUOTA: 500/1000"
// - [x] Refactor
//   - [x] better name for "bindless material" and move it out to lib
//   - [x] split spawn_chunk into two functions: one should spawn just the logic, the other should
//   be `load_chunk` which just loads the graphics; then do a `unload_chunk` which would unload the
//   graphics for the chunk; probably this will need each tile to have a reference to the chunk
//   entity and the chunk entity to have the coord in it
//   - [x] have a spawn_chunks_around_camera which just spawns the chunks and also a
//   load_chunks_around the camera which actually loads the chunk; then have a unload the chunks
//   for chunk that are not visible (maybe this will let us multithread the spawn)
//
// # Version 0.2
// - [x] Tile based map V2
//   - [x] create a system that can extend the map in any direction
//   - [x] implement loading and unloading tiles when scrolling trough the map
//   - [x] keep only 3x3 tilemaps around camera
//   - [x] keep the rest of the chunks loaded and updated but not shown
// - [ ] Resources
//   - [ ] use TileKind as the base of the tile: e.g Water, Land, BarrenLand
//   - [ ] think about this: implement something like DecorationKind for specifying the resources
//   that are on top of tiles
//   - [ ] implement Poisson disc distribution for nicer resource patches in a tile
//   - [ ] implement additional noise layer that will be used for each resource type
// - [ ] Pathfinding
// - [ ] Better Camera Using PanOrbitCameraPlugin for now, but we will need to create our own camera
// - [ ] plan for V3
//

fn main() {
    let mut app = App::new();

    #[cfg(feature = "debug")]
    app.add_plugins(DebugModePlugin);

    #[cfg(not(feature = "debug"))]
    app.add_plugins(DefaultPlugins);

    #[cfg(feature = "debug")]
    app.add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
        level: bevy::log::Level::DEBUG,
        ..default()
    }));

    app.add_plugins(PanOrbitCameraPlugin)
        .add_plugins(TerrainPlugin)
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .add_systems(OnEnter(GameStates::Playing), setup)
        .run();
}

fn setup(mut commands: Commands) {
    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
    ));
}
