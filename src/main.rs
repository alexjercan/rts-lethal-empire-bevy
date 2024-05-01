use std::collections::HashMap;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use itertools::Itertools;
use lethal_empire_bevy::helpers;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Perlin,
};

#[cfg(feature = "debug")]
use debug::DebugModePlugin;

// Features that I want in my game:
//
// # Version 0.1
// - [ ] Tile based map
//   - [x] create a perlin noise generator that can give us noise values for a given area (chunk)
//   - [ ] implement a mapper from noise values to tile types
//   - [ ] create a custom atlas with all the tile types and textures
//   - [ ] write a shader that takes in the tile types (RGB) and the atlas and renders the map
// - [ ] Resources
//   - [ ] implement a system that randomly generates resources on the map
//   - [ ] implement a new tile type for trees and rocks
//   - [ ] add models for trees and rocks and spawn them in the world
//   - [ ] think about a better way than just random for V2
// - [ ] Buildings
//   - [ ] extremely simple buildings that can be placed on the map and give us resources over time
// - [ ] Main Goal
//   - [ ] need to pay quota of resources to the Empire over time
//   - [ ] UI with the timer and quota needed and also how much we have
//   - "TIME LEFT: 10:00" "QUOTA: 500/1000"
//
// # Version 0.2
// - [ ] Tile based map V2
//   - [ ] create a system that can extend the map in any direction
//   - [ ] implement loading and unloading tiles when scrolling trough the map
// - [ ] Resources
// - [ ] Pathfinding
//

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(AssetCollection, Resource)]
struct GameAssets {}

#[derive(Resource, Deref)]
struct TerrainGenerator(Fbm<Perlin>);

impl Default for TerrainGenerator {
    fn default() -> Self {
        TerrainGenerator(
            Fbm::<Perlin>::new(0)
                .set_frequency(1.0)
                .set_persistence(0.5)
                .set_lacunarity(2.0)
                .set_octaves(14),
        )
    }
}

impl TerrainGenerator {
    fn generate(&self, center: IVec2, size: UVec2) -> Vec<f64> {
        PlaneMapBuilder::new(self.0.clone())
            .set_size(size.x as usize, size.y as usize)
            .set_x_bounds((center.x as f64) * 1.0 - 0.5, (center.x as f64) * 1.0 + 0.5)
            .set_y_bounds((center.y as f64) * 1.0 - 0.5, (center.y as f64) * 1.0 + 0.5)
            .build()
            .into_iter()
            .collect_vec()
    }
}

fn main() {
    let mut app = App::new();

    #[cfg(feature = "debug")]
    app.add_plugins(DebugModePlugin);

    app.add_plugins(DefaultPlugins)
        // TODO: Using PanOrbitCameraPlugin for now, but we will need to create our own camera
        .add_plugins(PanOrbitCameraPlugin)
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .init_resource::<TerrainGenerator>()
        .add_systems(OnEnter(GameStates::Playing), setup)
        .add_systems(Update, update.run_if(in_state(GameStates::Playing)))
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

    // map
    let map_size = UVec2::splat(128);
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = HashMap::<UVec2, Entity>::new();
    commands.entity(tilemap_entity).with_children(|parent| {
        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let tile_coord = UVec2::new(x, y);
                let tile_entity = parent.spawn((TileCoord(tile_coord),)).id();
                tile_storage.insert(UVec2::new(x, y), tile_entity);
            }
        }
    });
    let tile_size = Vec2::splat(16.0);
    commands.entity(tilemap_entity).insert((
        TilemapSize(map_size),
        TilemapStorage(tile_storage),
        TilemapTileSize(tile_size),
        helpers::geometry::get_tilemap_center_transform(&map_size, &tile_size, 0.0),
    ));
}

#[derive(Component, Deref)]
struct TileCoord(UVec2);

#[derive(Component, Deref)]
struct TilemapSize(UVec2);

#[derive(Component, Deref)]
struct TilemapStorage(HashMap<UVec2, Entity>);

#[derive(Component, Deref)]
struct TilemapTileSize(Vec2);

fn update(
    mut commands: Commands,
    terrain_generator: Res<TerrainGenerator>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let noisemap = terrain_generator.generate(IVec2::ZERO, UVec2::splat(128));

        println!("{:?}", noisemap);
    }
}

#[cfg(feature = "debug")]
mod debug {
    use bevy::prelude::*;

    use crate::GameStates;

    #[derive(Debug, Default)]
    pub(super) struct DebugModePlugin;

    impl Plugin for DebugModePlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, draw_cursor.run_if(in_state(GameStates::Playing)));
        }
    }

    fn draw_cursor(
        q_camera: Query<(&Camera, &GlobalTransform)>,
        windows: Query<&Window>,
        mut gizmos: Gizmos,
    ) {
        let (camera, camera_transform) = q_camera.single();

        let Some(cursor_position) = windows.single().cursor_position() else {
            return;
        };

        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

        let Some(distance) = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y)) else {
            return;
        };
        let point = ray.get_point(distance);

        gizmos.circle(point + Vec3::Y * 0.01, Direction3d::Y, 0.2, Color::WHITE);
    }
}
