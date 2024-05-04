use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use lethal_empire_bevy::{assets::GameAssets, states::GameStates, terrain::TerrainPlugin};

#[cfg(feature = "debug")]
mod debug;

#[cfg(feature = "debug")]
use debug::DebugModePlugin;

// # Version 0.2
// - [x] Tile based map V2
//   - [x] create a system that can extend the map in any direction
//   - [x] implement loading and unloading tiles when scrolling trough the map
//   - [x] keep only 3x3 tilemaps around camera
//   - [x] keep the rest of the chunks loaded and updated but not shown
// - [ ] Resources
//   - [x] use TileKind as the base of the tile: e.g Water, Land, BarrenLand
//   - [x] implement Poisson disc distribution for nicer resource patches in a tile
//   - [x] implement additional noise layer that will be used for each resource type
//   - [ ] do actually nicer resource patches
// - [ ] fix the seed (again...): use a rng to generate seeds
// - [ ] Buildings
//   - [ ] placing building on the map
//   - [ ] with units that can get resources from the map
// - [ ] Pathfinding
// - [ ] Main Goal
//   - [ ] need to pay quota of resources to the Empire over time
//   - [ ] UI with the timer and quota needed and also how much we have
//   - "TIME LEFT: 10:00" "QUOTA: 500/1000"
// - [ ] Better Camera: Using PanOrbitCameraPlugin for now, but we will need to create our own camera
// - [ ] plan for V3

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
