use bevy::prelude::*;

use crate::{assets::GameAssets, states::GameStates};

use self::{
    chunking::ChunkManager,
    helpers::geometry,
    materials::TerrainMaterial,
    resources::{ResourceKind, ResourceMapping, ResourcePlugin},
    tiles::{TileMapping, TilesPlugin},
};

mod resources;
mod tiles;

mod chunking;
mod helpers;
mod materials;

const SPAWN_CHUNK_RADIUS: usize = 8;
const LOAD_CHUNK_RADIUS: usize = 3;

#[derive(Component, Deref)]
struct ChunkCoord(IVec2);

#[derive(Component)]
struct ChunkHandledTiles;

#[derive(Component)]
struct ChunkHandledResources;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default())
            .add_plugins(TilesPlugin)
            .add_plugins(ResourcePlugin)
            .init_resource::<ChunkManager>()
            .add_systems(
                Update,
                (
                    spawn_chunks_around_camera,
                    load_chunks_around_camera,
                    unload_chunks_outside_camera,
                    handle_chunks_tiles,
                    handle_chunks_resources,
                )
                    .run_if(in_state(GameStates::Playing)),
            );
    }
}

fn handle_chunks_tiles(
    mut commands: Commands,
    chunk_manager: Res<ChunkManager>,
    q_chunks: Query<(Entity, &TileMapping), (With<ChunkCoord>, Without<ChunkHandledTiles>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    game_assets: Res<GameAssets>,
) {
    let chunk_size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();

    for (entity, tile_mapping) in q_chunks.iter() {
        let chunk_mesh = meshes.add(Plane3d::default().mesh().size(
            tile_size.x * chunk_size.x as f32,
            tile_size.y * chunk_size.y as f32,
        ));

        let chunk_material = materials.add(TerrainMaterial::new(
            chunk_size,
            game_assets.tiles.clone(),
            tile_mapping.iter().map(|kind| *kind as u32).collect(),
        ));

        commands
            .entity(entity)
            .insert((chunk_mesh, chunk_material, ChunkHandledTiles));
    }
}

fn handle_chunks_resources(
    mut commands: Commands,
    chunk_manager: Res<ChunkManager>,
    q_chunks: Query<
        (Entity, &TileMapping, &ResourceMapping),
        (With<ChunkCoord>, Without<ChunkHandledResources>),
    >,
    game_assets: Res<GameAssets>,
) {
    let chunk_size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();

    for (entity, tile_mapping, resource_mapping) in q_chunks.iter() {
        commands
            .entity(entity)
            .insert(ChunkHandledResources)
            .with_children(|parent| {
                // TODO: nicer assets based on tile kind
                for (index, (resource, _tile)) in
                    resource_mapping.iter().zip(tile_mapping.iter()).enumerate()
                {
                    let tile_coord =
                        UVec2::new(index as u32 % chunk_size.x, index as u32 / chunk_size.x);
                    match resource {
                        ResourceKind::None => (),
                        ResourceKind::Tree => {
                            parent.spawn((SceneBundle {
                                scene: game_assets.tree.clone(),
                                transform: helpers::geometry::get_tile_coord_transform(
                                    &tile_coord,
                                    &chunk_size,
                                    &tile_size,
                                    0.0,
                                )
                                .with_scale(Vec3::splat(4.0)),
                                ..Default::default()
                            },));
                        }
                        ResourceKind::Rock => {
                            parent.spawn((SceneBundle {
                                scene: game_assets.rock.clone(),
                                transform: helpers::geometry::get_tile_coord_transform(
                                    &tile_coord,
                                    &chunk_size,
                                    &tile_size,
                                    0.0,
                                )
                                .with_scale(Vec3::splat(8.0)),
                                ..Default::default()
                            },));
                        }
                    }
                }
            });
    }
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    q_camera: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let chunk_size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();

    for transform in q_camera.iter() {
        let camera_chunk_pos = geometry::world_pos_to_chunk_coord(
            &transform.translation.xz(),
            &chunk_manager.size(),
            &chunk_manager.tile_size(),
        );

        for y in (camera_chunk_pos.y - SPAWN_CHUNK_RADIUS as i32)
            ..=(camera_chunk_pos.y + SPAWN_CHUNK_RADIUS as i32)
        {
            for x in (camera_chunk_pos.x - SPAWN_CHUNK_RADIUS as i32)
                ..=(camera_chunk_pos.x + SPAWN_CHUNK_RADIUS as i32)
            {
                let coord = IVec2::new(x, y);
                if !chunk_manager.contains(&coord) {
                    debug!("Spawning chunk at {:?}", coord);

                    let chunk_entity = commands.spawn_empty().id();
                    chunk_manager.insert(coord, chunk_entity);
                    commands.entity(chunk_entity).insert((
                        ChunkCoord(coord),
                        SpatialBundle {
                            transform: geometry::get_chunk_coord_transform(
                                &coord,
                                &chunk_size,
                                &tile_size,
                                0.0,
                            ),
                            visibility: Visibility::Hidden,
                            ..default()
                        },
                    ));
                }
            }
        }
    }
}

fn load_chunks_around_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut q_chunks: Query<&mut Visibility, With<ChunkCoord>>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = geometry::world_pos_to_chunk_coord(
            &transform.translation.xz(),
            &chunk_manager.size(),
            &chunk_manager.tile_size(),
        );

        for y in (camera_chunk_pos.y - LOAD_CHUNK_RADIUS as i32)
            ..=(camera_chunk_pos.y + LOAD_CHUNK_RADIUS as i32)
        {
            for x in (camera_chunk_pos.x - LOAD_CHUNK_RADIUS as i32)
                ..=(camera_chunk_pos.x + LOAD_CHUNK_RADIUS as i32)
            {
                let coord = IVec2::new(x, y);
                if !chunk_manager.loaded(&coord) {
                    if let Some(chunk_entity) = chunk_manager.get(&coord) {
                        if let Ok(mut visibility) = q_chunks.get_mut(*chunk_entity) {
                            debug!("Load chunk at {:?}", coord);

                            *visibility = Visibility::Visible;
                            chunk_manager.load(coord);
                        }
                    }
                }
            }
        }
    }
}

fn unload_chunks_outside_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut q_chunks: Query<&mut Visibility, With<ChunkCoord>>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = geometry::world_pos_to_chunk_coord(
            &transform.translation.xz(),
            &chunk_manager.size(),
            &chunk_manager.tile_size(),
        );

        for coord in chunk_manager.out_range(&camera_chunk_pos, LOAD_CHUNK_RADIUS as i32) {
            if let Some(chunk_entity) = chunk_manager.get(&coord) {
                if let Ok(mut visibility) = q_chunks.get_mut(*chunk_entity) {
                    debug!("Unload chunk at {:?}", coord);

                    *visibility = Visibility::Hidden;
                    chunk_manager.unload(coord);
                }
            }
        }
    }
}
