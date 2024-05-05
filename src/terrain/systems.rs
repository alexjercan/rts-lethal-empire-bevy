use rand::{rngs::StdRng, SeedableRng};

use crate::{core::{GameAssets, Obstacle}, helpers::{self, sampling::disc::PoissonDiscSampler}};

use bevy::{
    ecs::system::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool},
};

use super::{
    ChunkCoord, ChunkHandledResources, ChunkHandledTiles, ChunkManager, ComputeResourceMapping,
    ComputeTileMapping, ResourceGenerator, ResourceKind, ResourceMapping, TerrainGenerator,
    TerrainMaterial, TerrainSeed, TileCoord, TileKind, TileMapping, LOAD_CHUNK_RADIUS,
    SPAWN_CHUNK_RADIUS,
};

pub fn handle_chunks_tiles(
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

pub fn handle_chunks_resources(
    mut commands: Commands,
    chunk_manager: Res<ChunkManager>,
    q_chunks: Query<
        (Entity, &ChunkCoord, &TileMapping, &ResourceMapping),
        Without<ChunkHandledResources>,
    >,
    game_assets: Res<GameAssets>,
    terrain_seed: Res<TerrainSeed>,
) {
    let chunk_size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();

    for (entity, chunk_coord, tile_mapping, resource_mapping) in q_chunks.iter() {
        commands
            .entity(entity)
            .insert(ChunkHandledResources)
            .with_children(|parent| {
                for (index, (resource, tile)) in
                    resource_mapping.iter().zip(tile_mapping.iter()).enumerate()
                {
                    let tile_coord = helpers::geometry::index_to_tile_coord(index, &chunk_size);
                    let global_coord = helpers::geometry::tile_coord_to_global_coord(
                        &tile_coord,
                        chunk_coord,
                        &chunk_size,
                    );
                    let tile_offset = helpers::geometry::tile_coord_to_world_off(
                        &tile_coord,
                        &chunk_size,
                        &tile_size,
                    );
                    let tile_seed = helpers::hash::seed_from_coord(**terrain_seed, &global_coord);

                    match (resource, tile) {
                        (_, TileKind::Water) => (),
                        (ResourceKind::None, _) => (),
                        (ResourceKind::Tree, TileKind::Grass) => {
                            let points = PoissonDiscSampler::new(tile_seed)
                                .with_radius(8.0)
                                .with_size(tile_size)
                                .with_k(30)
                                .sample();

                            for point in points {
                                let translation =
                                    (tile_offset + point - tile_size / 2.0).extend(0.0).xzy();

                                parent.spawn((
                                    TileCoord(tile_coord),
                                    Obstacle,
                                    SceneBundle {
                                        scene: game_assets.tree.clone(),
                                        transform: Transform::from_translation(translation)
                                            .with_scale(Vec3::splat(4.0)),
                                        ..default()
                                    },
                                ));
                            }
                        }
                        (ResourceKind::Tree, TileKind::Barren) => {
                            let points = PoissonDiscSampler::new(tile_seed)
                                .with_radius(12.0)
                                .with_size(tile_size)
                                .with_k(30)
                                .sample();

                            for point in points {
                                let translation =
                                    (tile_offset + point - tile_size / 2.0).extend(0.0).xzy();

                                parent.spawn((
                                    TileCoord(tile_coord),
                                    Obstacle,
                                    SceneBundle {
                                        scene: game_assets.tree_dead.clone(),
                                        transform: Transform::from_translation(translation)
                                            .with_scale(Vec3::splat(4.0)),
                                        ..default()
                                    },
                                ));
                            }
                        }
                        (ResourceKind::Rock, _) => {
                            let mut rng = StdRng::seed_from_u64(tile_seed);
                            let points = PoissonDiscSampler::new(tile_seed)
                                .with_radius(12.0)
                                .with_size(tile_size)
                                .with_k(30)
                                .sample();

                            for point in points {
                                let rotation_y = helpers::hash::random_angle(&mut rng);
                                let translation =
                                    (tile_offset + point - tile_size / 2.0).extend(0.0).xzy();

                                parent.spawn((
                                    TileCoord(tile_coord),
                                    Obstacle,
                                    SceneBundle {
                                        scene: game_assets.rock.clone(),
                                        transform: Transform::from_translation(translation)
                                            .with_scale(Vec3::splat(16.0))
                                            .with_rotation(Quat::from_rotation_y(rotation_y)),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    }
                }
            });
    }
}

pub fn spawn_chunks_around_camera(
    mut commands: Commands,
    q_camera: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let chunk_size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();

    for transform in q_camera.iter() {
        let camera_chunk_pos = helpers::geometry::world_pos_to_chunk_coord(
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
                    let translation = helpers::geometry::chunk_coord_to_world_pos(
                        &coord,
                        &chunk_size,
                        &tile_size,
                    )
                    .extend(0.0)
                    .xzy();

                    commands.entity(chunk_entity).insert((
                        ChunkCoord(coord),
                        SpatialBundle {
                            transform: Transform::from_translation(translation),
                            visibility: Visibility::Hidden,
                            ..default()
                        },
                    ));
                }
            }
        }
    }
}

pub fn load_chunks_around_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut q_chunks: Query<&mut Visibility, With<ChunkCoord>>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = helpers::geometry::world_pos_to_chunk_coord(
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

pub fn unload_chunks_outside_camera(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut q_chunks: Query<&mut Visibility, With<ChunkCoord>>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = helpers::geometry::world_pos_to_chunk_coord(
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

pub fn generate_terrain_task(
    mut commands: Commands,
    terrain_generator: Res<TerrainGenerator>,
    q_chunks: Query<(Entity, &ChunkCoord), (Without<TileMapping>, Without<ComputeTileMapping>)>,
    chunk_manager: Res<ChunkManager>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (chunk, coord) in q_chunks.iter() {
        let coord = **coord;
        debug!("Spawning tile mapping for chunk at {:?}", coord);

        let chunk_size = chunk_manager.size();
        let terrain_generator = terrain_generator.clone();

        let task = thread_pool.spawn(async move {
            let span = info_span!("generate tile mapping").entered();
            let mapping = terrain_generator.generate(coord, chunk_size);
            span.exit();

            let mut command_queue = CommandQueue::default();
            command_queue.push(move |world: &mut World| {
                world
                    .entity_mut(chunk)
                    .insert(TileMapping(mapping))
                    .remove::<ComputeTileMapping>();
            });

            command_queue
        });

        commands.entity(chunk).insert(ComputeTileMapping(task));
    }
}

pub fn handle_generate_terrain_task(
    mut commands: Commands,
    mut tasks: Query<&mut ComputeTileMapping>,
) {
    for mut task in &mut tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
    }
}

pub fn generate_resource_task(
    mut commands: Commands,
    resource_generator: Res<ResourceGenerator>,
    q_chunks: Query<
        (Entity, &ChunkCoord),
        (Without<ResourceMapping>, Without<ComputeResourceMapping>),
    >,
    chunk_manager: Res<ChunkManager>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (chunk, coord) in q_chunks.iter() {
        let coord = **coord;
        debug!("Spawning tile mapping for chunk at {:?}", coord);

        let chunk_size = chunk_manager.size();
        let resource_generator = resource_generator.clone();

        let task = thread_pool.spawn(async move {
            let span = info_span!("generate resource mapping").entered();
            let mapping = resource_generator.generate(coord, chunk_size);
            span.exit();

            let mut command_queue = CommandQueue::default();
            command_queue.push(move |world: &mut World| {
                world
                    .entity_mut(chunk)
                    .insert(ResourceMapping(mapping))
                    .remove::<ComputeResourceMapping>();
            });

            command_queue
        });

        commands.entity(chunk).insert(ComputeResourceMapping(task));
    }
}

pub fn handle_generate_resource_task(
    mut commands: Commands,
    mut tasks: Query<&mut ComputeResourceMapping>,
) {
    for mut task in &mut tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
    }
}
