use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use itertools::Itertools;
use lethal_empire_bevy::{helpers, materials::tilemap::TilemapMaterial};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Perlin,
};

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
// - [ ] Refactor
//   - [x] better name for "bindless material" and move it out to lib
//   - [ ] split spawn_chunk into two functions: one should spawn just the logic, the other should
//   be `load_chunk` which just loads the graphics; then do a `unload_chunk` which would unload the
//   graphics for the chunk; probably this will need each tile to have a reference to the chunk
//   entity and the chunk entity to have the coord in it
//   - [ ] have a spawn_chunks_around_camera which just spawns the chunks and also a
//   load_chunks_around the camera which actually loads the chunk; then have a unload the chunks
//   for chunk that are not visible (maybe this will let us multithread the spawn)
//
// # Version 0.2
// - [ ] Tile based map V2
//   - [x] create a system that can extend the map in any direction
//   - [ ] implement loading and unloading tiles when scrolling trough the map
//   - [ ] keep only 3x3 tilemaps around camera
//   - [ ] keep the rest of the chunks loaded and updated but not shown
// - [ ] Resources
//   - [ ] implement Poisson disc distribution for nicer resource patches in a tile
//   - [ ] implement additional noise layer that will be used for each resource type
// - [ ] Pathfinding
//

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(
        paths(
            "textures/tiles/water.png",
            "textures/tiles/grass.png",
            "textures/tiles/forest.png",
            "textures/tiles/rock.png"
        ),
        collection(typed)
    )]
    tiles: Vec<Handle<Image>>,
    #[asset(path = "models/lowpoly_tree/tree.gltf#Scene0")]
    tree: Handle<Scene>,
    #[asset(path = "models/lowpoly_stone/stone_tallA.glb#Scene0")]
    rock: Handle<Scene>,
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
enum TileKind {
    #[default]
    Water,
    Grass,
    Forest,
    Rock,
}

impl TileKind {
    fn from_noise(noise: f64) -> Self {
        match noise {
            n if n < 0.0 => TileKind::Water,
            n if n < 0.2 => TileKind::Grass,
            n if n < 0.4 => TileKind::Forest,
            _ => TileKind::Rock,
        }
    }
}

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
    fn generate(&self, coord: IVec2, size: UVec2) -> Vec<f64> {
        PlaneMapBuilder::new(self.0.clone())
            .set_size(size.x as usize, size.y as usize)
            .set_x_bounds((coord.x as f64) * 1.0 - 0.5, (coord.x as f64) * 1.0 + 0.5)
            .set_y_bounds((coord.y as f64) * 1.0 - 0.5, (coord.y as f64) * 1.0 + 0.5)
            .build()
            .into_iter()
            .collect_vec()
    }
}

#[derive(Default, Debug, Resource, Deref, DerefMut)]
struct ChunkManager(HashSet<IVec2>);

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

    app
        // TODO: Using PanOrbitCameraPlugin for now, but we will need to create our own camera
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(MaterialPlugin::<TilemapMaterial>::default())
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .init_resource::<TerrainGenerator>()
        .init_resource::<ChunkManager>()
        .add_systems(OnEnter(GameStates::Playing), setup)
        .add_systems(
            Update,
            (spawn_chunks_around_camera).run_if(in_state(GameStates::Playing)),
        )
        .run();
}

const TILEMAP_SIZE: usize = 128;
const TILEMAP_TILE_SIZE: f32 = 16.0;
const TILEMAP_CHUNK_RADIUS: usize = 2;

fn spawn_chunk(
    coord: IVec2,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<TilemapMaterial>>,
    game_assets: &Res<GameAssets>,
    terrain_generator: &Res<TerrainGenerator>,
) {
    let noisemap = terrain_generator.generate(coord, UVec2::splat(TILEMAP_SIZE as u32));

    let map_size = UVec2::splat(TILEMAP_SIZE as u32);
    let tile_size = Vec2::splat(TILEMAP_TILE_SIZE);

    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = HashMap::<UVec2, Entity>::new();
    let mut mapping = vec![];
    commands.entity(tilemap_entity).with_children(|parent| {
        for y in 0..map_size.y {
            for x in 0..map_size.x {
                let tile_coord = UVec2::new(x, y);
                let noise = noisemap[map_size.x as usize * y as usize + x as usize];
                let tile_kind = TileKind::from_noise(noise);

                mapping.push(tile_kind);
                let tile_entity = parent.spawn((TileCoord(tile_coord), tile_kind)).id();
                tile_storage.insert(UVec2::new(x, y), tile_entity);
            }
        }
    });

    let chunk_pos = helpers::geometry::chunk_coord_to_world_pos(&coord, &map_size, &tile_size);
    info!("Chunk pos: {:?}", chunk_pos);
    for (tile_coord, _) in tile_storage.iter() {
        let index = map_size.x as usize * tile_coord.y as usize + tile_coord.x as usize;
        let tile_kind = mapping[index];

        let tile_off = helpers::geometry::tile_coord_to_world_offset(
            &tile_coord.as_ivec2(),
            &map_size,
            &tile_size,
        );
        let position = chunk_pos + tile_off + tile_size / 2.0;

        match tile_kind {
            TileKind::Water => (),
            TileKind::Grass => (),
            TileKind::Forest => {
                commands.spawn((SceneBundle {
                    scene: game_assets.tree.clone(),
                    transform: Transform::from_translation(position.extend(0.0).xzy())
                        .with_scale(Vec3::splat(4.0)),
                    ..Default::default()
                },));
            }
            TileKind::Rock => {
                commands.spawn((SceneBundle {
                    scene: game_assets.rock.clone(),
                    transform: Transform::from_translation(position.extend(0.0).xzy())
                        .with_scale(Vec3::splat(8.0)),
                    ..Default::default()
                },));
            }
        }
    }

    commands.entity(tilemap_entity).insert((
        TilemapSize(map_size),
        TilemapStorage(tile_storage),
        TilemapTileSize(tile_size),
        MaterialMeshBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(
                tile_size.x * map_size.x as f32,
                tile_size.y * map_size.y as f32,
            )),
            material: materials.add(TilemapMaterial::new(
                TILEMAP_SIZE as u32,
                game_assets.tiles.clone(),
                mapping.iter().map(|kind| *kind as u32).collect(),
            )),
            transform: helpers::geometry::get_tilemap_coord_transform(
                &coord, &map_size, &tile_size, 0.0,
            ),
            ..Default::default()
        },
    ));
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

fn spawn_chunks_around_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
    game_assets: Res<GameAssets>,
    terrain_generator: Res<TerrainGenerator>,
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = helpers::geometry::world_pos_to_chunk_coord(
            &transform.translation.xz(),
            &UVec2::splat(TILEMAP_SIZE as u32),
            &Vec2::splat(TILEMAP_TILE_SIZE),
        );

        for y in (camera_chunk_pos.y - TILEMAP_CHUNK_RADIUS as i32)
            ..=(camera_chunk_pos.y + TILEMAP_CHUNK_RADIUS as i32)
        {
            for x in (camera_chunk_pos.x - TILEMAP_CHUNK_RADIUS as i32)
                ..=(camera_chunk_pos.x + TILEMAP_CHUNK_RADIUS as i32)
            {
                if !chunk_manager.contains(&IVec2::new(x, y)) {
                    debug!("Spawning chunk at {:?}", IVec2::new(x, y));
                    chunk_manager.insert(IVec2::new(x, y));
                    spawn_chunk(
                        IVec2::new(x, y),
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &game_assets,
                        &terrain_generator,
                    );
                }
            }
        }
    }
}

#[derive(Component, Deref)]
struct TileCoord(UVec2);

#[derive(Component, Deref)]
struct TilemapSize(UVec2);

#[derive(Component, Deref)]
struct TilemapStorage(HashMap<UVec2, Entity>);

#[derive(Component, Deref)]
struct TilemapTileSize(Vec2);

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
