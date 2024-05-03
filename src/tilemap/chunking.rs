use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::{assets::GameAssets, states::GameStates};

use super::{helpers::geometry, materials::TilemapMaterial, terrain::{TerrainGenerator, TerrainKind}};

pub struct ChunkingPlugin;

impl Plugin for ChunkingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MaterialPlugin::<TilemapMaterial>::default())
            .init_resource::<TerrainGenerator>()
            .init_resource::<ChunkManager>()
            .add_systems(
                Update,
                (spawn_chunks_around_camera, load_chunks_around_camera, unload_chunks_outside_camera).run_if(in_state(GameStates::Playing)),
            );
    }
}

const TILEMAP_SIZE: usize = 128;
const TILEMAP_TILE_SIZE: f32 = 16.0;
const TILEMAP_CHUNK_RADIUS: usize = 2;

#[derive(Component)]
struct TileKind(TerrainKind);

#[derive(Component, Deref)]
struct TileCoord(UVec2);

#[derive(Component, Deref)]
struct TileParent(Entity);

#[derive(Component, Deref)]
struct TilemapSize(UVec2);

#[derive(Component, Deref)]
struct TilemapStorage(HashMap<UVec2, Entity>);

#[derive(Component, Deref)]
struct TilemapTileSize(Vec2);

#[derive(Component, Deref)]
struct TilemapCoord(IVec2);


#[derive(Debug, Resource)]
struct ChunkManager {
    size: UVec2,
    tile_size: Vec2,
    chunks: HashMap<IVec2, Entity>,
    loaded: HashSet<IVec2>,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            size: UVec2::splat(TILEMAP_SIZE as u32),
            tile_size: Vec2::splat(TILEMAP_TILE_SIZE),
            chunks: HashMap::new(),
            loaded: HashSet::new(),
        }
    }
}

impl ChunkManager {
    fn spawn(
        &mut self,
        coord: IVec2,
        commands: &mut Commands,
        terrain_generator: &Res<TerrainGenerator>,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<TilemapMaterial>>,
        game_assets: &Res<GameAssets>,
    ) {
        let tilemap_entity = commands.spawn_empty().id();

        let mapping = terrain_generator.generate(coord, self.size);

        let mut tile_storage = HashMap::<UVec2, Entity>::new();
        commands.entity(tilemap_entity).with_children(|parent| {
            for y in 0..self.size.y {
                for x in 0..self.size.x {
                    let tile_coord = UVec2::new(x, y);
                    let tile_kind = mapping[self.size.x as usize * y as usize + x as usize];

                    let mut tile = parent.spawn((
                        TileCoord(tile_coord),
                        TileParent(tilemap_entity),
                        TileKind(tile_kind),
                        SpatialBundle {
                            transform: geometry::get_tile_coord_transform(
                                &tile_coord.as_ivec2(),
                                &self.size,
                                &self.tile_size,
                                0.0,
                            ),
                            ..default()
                        },
                    ));
                    let tile_entity = tile.id();
                    tile_storage.insert(tile_coord, tile_entity);

                    tile.with_children(|parent| {
                        match tile_kind {
                            TerrainKind::Water => (),
                            TerrainKind::Grass => (),
                            TerrainKind::Forest => {
                                parent.spawn((SceneBundle {
                                    scene: game_assets.tree.clone(),
                                    transform: Transform::from_translation(
                                        Vec3::ZERO,
                                    )
                                    .with_scale(Vec3::splat(4.0)),
                                    ..Default::default()
                                },));
                            }
                            TerrainKind::Rock => {
                                parent.spawn((SceneBundle {
                                    scene: game_assets.rock.clone(),
                                    transform: Transform::from_translation(
                                        Vec3::ZERO,
                                    )
                                    .with_scale(Vec3::splat(8.0)),
                                    ..Default::default()
                                },));
                            }
                        }
                    });
                }
            }
        });

        commands.entity(tilemap_entity).insert((
            TilemapSize(self.size),
            TilemapStorage(tile_storage),
            TilemapTileSize(self.tile_size),
            TilemapCoord(coord),
            MaterialMeshBundle {
                mesh: meshes.add(Plane3d::default().mesh().size(
                    self.tile_size.x * self.size.x as f32,
                    self.tile_size.y * self.size.y as f32,
                )),
                material: materials.add(TilemapMaterial::new(
                    self.size,
                    game_assets.tiles.clone(),
                    mapping.iter().map(|kind| *kind as u32).collect(),
                )),
                transform: geometry::get_tilemap_coord_transform(
                    &coord,
                    &self.size,
                    &self.tile_size,
                    0.0,
                ),
                visibility: Visibility::Hidden,
                ..default()
            },
        ));

        self.chunks.insert(coord, tilemap_entity);
    }

    fn load(&mut self, coord: IVec2, q_tilemap: &mut Query<&mut Visibility, With<TilemapCoord>>) {
        if let Some(tilemap_entity) = self.chunks.get(&coord) {
            if let Ok(mut visibility) = q_tilemap.get_mut(*tilemap_entity) {
                *visibility = Visibility::Visible;

                self.loaded.insert(coord);
            }
        }
    }

    fn unload(&mut self, coord: IVec2, q_tilemap: &mut Query<&mut Visibility, With<TilemapCoord>>) {
        if let Some(tilemap_entity) = self.chunks.get(&coord) {
            if let Ok(mut visibility) = q_tilemap.get_mut(*tilemap_entity) {
                *visibility = Visibility::Hidden;

                self.loaded.remove(&coord);
            }
        }
    }

    fn contains(&self, coord: &IVec2) -> bool {
        self.chunks.contains_key(coord)
    }

    fn loaded(&self, coord: &IVec2) -> bool {
        self.loaded.contains(coord)
    }
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    terrain_generator: Res<TerrainGenerator>,
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
    game_assets: Res<GameAssets>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = geometry::world_pos_to_chunk_coord(
            &transform.translation.xz(),
            &chunk_manager.size,
            &chunk_manager.tile_size,
        );

        for y in (camera_chunk_pos.y - TILEMAP_CHUNK_RADIUS as i32)
            ..=(camera_chunk_pos.y + TILEMAP_CHUNK_RADIUS as i32)
        {
            for x in (camera_chunk_pos.x - TILEMAP_CHUNK_RADIUS as i32)
                ..=(camera_chunk_pos.x + TILEMAP_CHUNK_RADIUS as i32)
            {
                if !chunk_manager.contains(&IVec2::new(x, y)) {
                    debug!("Spawning chunk at {:?}", IVec2::new(x, y));
                    chunk_manager.spawn(
                        IVec2::new(x, y),
                        &mut commands,
                        &terrain_generator,
                        &mut meshes,
                        &mut materials,
                        &game_assets,
                    );
                }
            }
        }
    }
}

fn load_chunks_around_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut q_tilemap: Query<&mut Visibility, With<TilemapCoord>>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = geometry::world_pos_to_chunk_coord(
            &transform.translation.xz(),
            &chunk_manager.size,
            &chunk_manager.tile_size,
        );

        for y in (camera_chunk_pos.y - TILEMAP_CHUNK_RADIUS as i32)
            ..=(camera_chunk_pos.y + TILEMAP_CHUNK_RADIUS as i32)
        {
            for x in (camera_chunk_pos.x - TILEMAP_CHUNK_RADIUS as i32)
                ..=(camera_chunk_pos.x + TILEMAP_CHUNK_RADIUS as i32)
            {
                if chunk_manager.contains(&IVec2::new(x, y))
                    && !chunk_manager.loaded(&IVec2::new(x, y))
                {
                    debug!("Load chunk at {:?}", IVec2::new(x, y));
                    chunk_manager.load(IVec2::new(x, y), &mut q_tilemap);
                }
            }
        }
    }
}

fn unload_chunks_outside_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut q_tilemap: Query<&mut Visibility, With<TilemapCoord>>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = geometry::world_pos_to_chunk_coord(
            &transform.translation.xz(),
            &chunk_manager.size,
            &chunk_manager.tile_size,
        );

        let mut to_unload = vec![];
        for coord in chunk_manager.loaded.iter() {
            if (camera_chunk_pos - *coord).abs().max_element() > TILEMAP_CHUNK_RADIUS as i32 {
                debug!("Unload chunk at {:?}", coord);
                to_unload.push(*coord);
            }
        }

        for coord in to_unload {
            chunk_manager.unload(coord, &mut q_tilemap);
        }
    }
}
