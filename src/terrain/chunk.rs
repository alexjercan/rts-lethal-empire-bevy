use bevy::{
    ecs::system::CommandQueue,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use noise::{
    core::worley::{distance_functions, ReturnType},
    utils::{ColorGradient, ImageRenderer, NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Perlin, Worley,
};

use crate::{terrain::ResourcePiece, GameAssets, GameStates};

use super::{poisson_disc::PoissonSampler, TerrainConfig};

#[derive(Component)]
pub(super) struct Chunk;

#[derive(Component, Deref)]
pub(super) struct ChunkCoord(pub IVec2);

#[derive(Component, Deref)]
struct ChunkNoiseMap(NoiseMapData);

#[derive(Component, Deref, DerefMut)]
struct ComputeNoiseMap(Task<CommandQueue>);

#[derive(Component)]
struct ChunkRender;

#[derive(Component)]
struct ResourcePosition(Vec2);

#[derive(Component)]
struct ResourceNoise(f64);

#[derive(Component)]
struct TreeResource;

#[derive(Component)]
struct RockResource;

#[derive(Component)]
struct ResourceRender;

struct TreeData {
    position: Vec2,
    noise: f64,
}

struct RockData {
    position: Vec2,
    noise: f64,
}

pub(super) struct NoiseMapData {
    size: UVec2,
    border_value: f64,
    map: Vec<f64>,
}

impl From<NoiseMap> for NoiseMapData {
    fn from(value: NoiseMap) -> Self {
        let size = value.size();
        let border_value = value.border_value();
        let map = value.into_iter().collect::<Vec<_>>();
        return NoiseMapData {
            size: UVec2::new(size.0 as u32, size.1 as u32),
            border_value,
            map,
        };
    }
}

impl<'a> Into<NoiseMap> for &'a NoiseMapData {
    fn into(self) -> NoiseMap {
        let mut noisemap = NoiseMap::new(self.size.x as usize, self.size.y as usize);
        noisemap
            .iter_mut()
            .zip(self.map.iter())
            .for_each(|(a, &b)| *a = b);
        return noisemap.set_border_value(self.border_value);
    }
}

// TODO: split this into better components
#[derive(Debug, Clone, Copy, Resource)]
pub(super) struct ChunkCPUConfig {
    terrain_frequency: f64,
    forest_frequency: f64,
    rock_frequency: f64,
    persistence: f64,
    lacunarity: f64,
    octaves: u32,
    scale: f32,
    bounds_interval: f64,
    forest_min_noise: f64,
    forest_max_noise: f64,
    forest_tree_radius: f32,
    forest_discard_threshold: f64,
    forest_k: u32,
    forest_threshold_noise: f64,
    rock_min_noise: f64,
    rock_max_noise: f64,
    rock_radius: f32,
    rock_discard_threshold: f64,
    rock_k: u32,
}

impl Default for ChunkCPUConfig {
    fn default() -> Self {
        ChunkCPUConfig {
            terrain_frequency: 1.0,
            forest_frequency: 10.0,
            rock_frequency: 1.0,
            persistence: 0.5,
            lacunarity: 2.0,
            octaves: 14,
            scale: 8.0,
            bounds_interval: 0.125,
            forest_min_noise: 4096.0 / 16384.0,
            forest_max_noise: 1.0,
            forest_tree_radius: 2.0,
            forest_discard_threshold: -0.2,
            forest_k: 30,
            forest_threshold_noise: 6144.0 / 16384.0,
            rock_min_noise: 1024.0 / 16384.0,
            rock_max_noise: 4096.0 / 16384.0,
            rock_radius: 4.0,
            rock_discard_threshold: -0.75,
            rock_k: 30,
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct ChunkCPUPlugin {
    config: ChunkCPUConfig,
}

impl Plugin for ChunkCPUPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.add_systems(
            Update,
            (
                spawn_compute_noisemap_task,
                join_compute_noisemap_task,
                handle_noisemap_render,
                handle_tree_render,
                handle_rock_render,
            )
                .run_if(in_state(GameStates::Playing)),
        );
    }
}

fn spawn_compute_noisemap_task(
    mut commands: Commands,
    q_chunk_entities: Query<
        (Entity, &ChunkCoord),
        (
            With<Chunk>,
            Without<ComputeNoiseMap>,
            Without<ChunkNoiseMap>,
        ),
    >,
    chunk_config: Res<ChunkCPUConfig>,
    terrain_config: Res<TerrainConfig>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (entity, chunk) in q_chunk_entities.iter() {
        let point = **chunk;
        let terrain_config = *terrain_config;
        let chunk_config = *chunk_config;

        let task = thread_pool.spawn(async move {
            let terrain_noisemap = terrain_generate(point, &terrain_config, &chunk_config);
            let forest_points =
                forest_generate(point, &terrain_noisemap, &terrain_config, &chunk_config);
            let rock_points =
                rock_generate(point, &terrain_noisemap, &terrain_config, &chunk_config);

            let mut command_queue = CommandQueue::default();

            command_queue.push(move |world: &mut World| {
                world
                    .entity_mut(entity)
                    .insert(ChunkNoiseMap(terrain_noisemap.into()))
                    .remove::<ComputeNoiseMap>();

                for point in forest_points {
                    world.spawn((
                        ResourcePosition(point.position),
                        ResourceNoise(point.noise),
                        ResourcePiece,
                        TreeResource,
                    ));
                }

                for point in rock_points {
                    world.spawn((
                        ResourcePosition(point.position),
                        ResourceNoise(point.noise),
                        ResourcePiece,
                        RockResource,
                    ));
                }
            });

            return command_queue;
        });

        commands.entity(entity).insert(ComputeNoiseMap(task));
    }
}

fn terrain_generate(
    coord: IVec2,
    terrain_config: &TerrainConfig,
    chunk_config: &ChunkCPUConfig,
) -> NoiseMap {
    let fbm = Fbm::<Perlin>::new(terrain_config.seed as u32)
        .set_frequency(chunk_config.terrain_frequency)
        .set_persistence(chunk_config.persistence)
        .set_lacunarity(chunk_config.lacunarity)
        .set_octaves(chunk_config.octaves as usize);

    let size = (terrain_config.chunk_size * chunk_config.scale) as usize;
    let noise_map = PlaneMapBuilder::new(fbm)
        .set_size(size, size)
        .set_x_bounds(
            (coord.x as f64) * chunk_config.bounds_interval - chunk_config.bounds_interval / 2.0,
            (coord.x as f64) * chunk_config.bounds_interval + chunk_config.bounds_interval / 2.0,
        )
        .set_y_bounds(
            (coord.y as f64) * chunk_config.bounds_interval - chunk_config.bounds_interval / 2.0,
            (coord.y as f64) * chunk_config.bounds_interval + chunk_config.bounds_interval / 2.0,
        )
        .build();

    return noise_map;
}

fn forest_generate(
    coord: IVec2,
    noisemap: &NoiseMap,
    terrain_config: &TerrainConfig,
    chunk_config: &ChunkCPUConfig,
) -> Vec<TreeData> {
    let worley = Worley::new(terrain_config.seed as u32)
        .set_distance_function(distance_functions::euclidean_squared)
        .set_return_type(ReturnType::Distance)
        .set_frequency(chunk_config.forest_frequency);

    let size = (terrain_config.chunk_size * chunk_config.scale) as usize;
    let mut resourcemap = PlaneMapBuilder::new(worley)
        .set_size(size, size)
        .set_x_bounds(
            (coord.x as f64) * chunk_config.bounds_interval - chunk_config.bounds_interval / 2.0,
            (coord.x as f64) * chunk_config.bounds_interval + chunk_config.bounds_interval / 2.0,
        )
        .set_y_bounds(
            (coord.y as f64) * chunk_config.bounds_interval - chunk_config.bounds_interval / 2.0,
            (coord.y as f64) * chunk_config.bounds_interval + chunk_config.bounds_interval / 2.0,
        )
        .build();

    resourcemap
        .iter_mut()
        .zip(noisemap.iter())
        .for_each(|(r, n)| {
            if *n < chunk_config.forest_min_noise || *n >= chunk_config.forest_max_noise {
                *r = -1.0;
            } else {
                if *r > chunk_config.forest_discard_threshold {
                    *r = -1.0
                } else {
                    *r = 0.5;
                }
            }
        });

    let forest_points = PoissonSampler::new(terrain_config.seed)
        .sample(
            chunk_config.forest_tree_radius,
            terrain_config.chunk_size,
            chunk_config.forest_k,
        )
        .into_iter()
        .filter(|p| {
            let p = *p * chunk_config.scale;
            let x = p.x as usize;
            let y = p.y as usize;
            resourcemap[(x, y)] > 0.0
        })
        .map(|p| TreeData {
            position: p + coord.as_vec2() * terrain_config.chunk_size
                - terrain_config.chunk_size / 2.0,
            noise: noisemap[(p.x as usize, p.y as usize)],
        })
        .collect();

    return forest_points;
}

fn rock_generate(
    coord: IVec2,
    noisemap: &NoiseMap,
    terrain_config: &TerrainConfig,
    chunk_config: &ChunkCPUConfig,
) -> Vec<RockData> {
    let worley = Worley::new(terrain_config.seed as u32)
        .set_distance_function(distance_functions::euclidean_squared)
        .set_return_type(ReturnType::Distance)
        .set_frequency(chunk_config.rock_frequency);

    let size = (terrain_config.chunk_size * chunk_config.scale) as usize;
    let mut resourcemap = PlaneMapBuilder::new(worley)
        .set_size(size, size)
        .set_x_bounds(
            (coord.x as f64) * chunk_config.bounds_interval - chunk_config.bounds_interval / 2.0,
            (coord.x as f64) * chunk_config.bounds_interval + chunk_config.bounds_interval / 2.0,
        )
        .set_y_bounds(
            (coord.y as f64) * chunk_config.bounds_interval - chunk_config.bounds_interval / 2.0,
            (coord.y as f64) * chunk_config.bounds_interval + chunk_config.bounds_interval / 2.0,
        )
        .build();

    resourcemap
        .iter_mut()
        .zip(noisemap.iter())
        .for_each(|(r, n)| {
            if *n < chunk_config.rock_min_noise || *n >= chunk_config.rock_max_noise {
                *r = -1.0;
            } else {
                if *r > chunk_config.rock_discard_threshold {
                    *r = -1.0
                } else {
                    *r = 0.5;
                }
            }
        });

    let rock_points = PoissonSampler::new(terrain_config.seed)
        .sample(
            chunk_config.rock_radius,
            terrain_config.chunk_size,
            chunk_config.rock_k,
        )
        .into_iter()
        .filter(|p| {
            let p = *p * chunk_config.scale;
            let x = p.x as usize;
            let y = p.y as usize;
            resourcemap[(x, y)] > 0.0
        })
        .map(|p| RockData {
            position: p + coord.as_vec2() * terrain_config.chunk_size
                - terrain_config.chunk_size / 2.0,
            noise: noisemap[(p.x as usize, p.y as usize)],
        })
        .collect();

    return rock_points;
}

fn join_compute_noisemap_task(
    mut commands: Commands,
    mut noise_map_tasks: Query<&mut ComputeNoiseMap, With<Chunk>>,
) {
    for mut task in noise_map_tasks.iter_mut() {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut **task)) {
            commands.append(&mut commands_queue);
        }
    }
}

fn handle_noisemap_render(
    mut commands: Commands,
    q_chunks: Query<(Entity, &ChunkCoord, &ChunkNoiseMap), (With<Chunk>, Without<ChunkRender>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    terrain_config: Res<TerrainConfig>,
) {
    for (entity, chunk, data) in q_chunks.iter() {
        let point = **chunk;

        let image = ImageRenderer::new()
            .set_gradient(ColorGradient::new().build_terrain_gradient())
            .render(&(&(**data)).into());

        let (width, height) = image.size();

        let image = Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                ..default()
            },
            TextureDimension::D2,
            image.into_iter().flatten().collect(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );

        let handler = images.add(image);
        let material = StandardMaterial::from(handler);

        commands
            .entity(entity)
            .insert((PbrBundle {
                mesh: meshes.add(
                    Plane3d::default()
                        .mesh()
                        .size(terrain_config.chunk_size, terrain_config.chunk_size),
                ),
                material: materials.add(material),
                transform: Transform::from_translation(
                    point.extend(0).xzy().as_vec3() * terrain_config.chunk_size,
                ),
                ..default()
            },))
            .insert(ChunkRender);
    }
}

fn handle_tree_render(
    mut commands: Commands,
    q_trees: Query<
        (Entity, &ResourcePosition, &ResourceNoise),
        (With<TreeResource>, Without<ResourceRender>),
    >,
    chunk_config: Res<ChunkCPUConfig>,
    game_assets: Res<GameAssets>,
) {
    for (tree, position, noise) in q_trees.iter() {
        commands.entity(tree).insert((
            SceneBundle {
                scene: if noise.0 < chunk_config.forest_threshold_noise {
                    game_assets.tree.clone()
                } else {
                    game_assets.tree_snow.clone()
                },
                transform: Transform::from_xyz(position.0.x, 0.0, position.0.y)
                    .with_scale(Vec3::splat(1.0)),
                ..Default::default()
            },
            ResourceRender,
        ));
    }
}

fn handle_rock_render(
    mut commands: Commands,
    q_rocks: Query<
        (Entity, &ResourcePosition, &ResourceNoise),
        (With<RockResource>, Without<ResourceRender>),
    >,
    game_assets: Res<GameAssets>,
) {
    for (rock, position, _noise) in q_rocks.iter() {
        let rocks = &game_assets.rocks;
        let index = rand::random::<usize>() % rocks.len();

        commands.entity(rock).insert((
            SceneBundle {
                scene: rocks[index].clone(),
                transform: Transform::from_xyz(position.0.x, 0.0, position.0.y)
                    .with_scale(Vec3::splat(5.0)),
                ..Default::default()
            },
            ResourceRender,
        ));
    }
}
