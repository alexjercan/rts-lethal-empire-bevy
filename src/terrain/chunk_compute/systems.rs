use std::ops::Deref;

use bevy::{
    ecs::system::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool},
};
use noise::{
    core::worley::{distance_functions, ReturnType},
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Perlin, Worley,
};

use crate::terrain::components::{ChunkData, RockData, TreeData};

use super::{
    components::ComputeNoiseMap, poisson_disc::PoissonSampler, resources::ChunkComputeCPUConfig,
    Chunk, ChunkCoord, TerrainConfig,
};

pub(super) fn spawn_noise_map_tasks(
    mut commands: Commands,
    q_chunk_entities: Query<
        (Entity, &ChunkCoord),
        (With<Chunk>, Without<ChunkData>, Without<ComputeNoiseMap>),
    >,
    chunk_config: Res<ChunkComputeCPUConfig>,
    terrain_config: Res<TerrainConfig>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (entity, chunk) in q_chunk_entities.iter() {
        let point = chunk.0;
        let terrain_config = *terrain_config.deref();
        let chunk_config = *chunk_config.deref();

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
                    .insert(ChunkData {
                        terrain: terrain_noisemap.into(),
                        forest: forest_points,
                        rocks: rock_points,
                    })
                    .remove::<ComputeNoiseMap>();
            });

            return command_queue;
        });

        debug!("Creating terrain generation task for chunk {:?}", point);

        commands.entity(entity).insert(ComputeNoiseMap(task));
    }
}

fn terrain_generate(
    coord: IVec2,
    terrain_config: &TerrainConfig,
    chunk_config: &ChunkComputeCPUConfig,
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
    chunk_config: &ChunkComputeCPUConfig,
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
    chunk_config: &ChunkComputeCPUConfig,
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

pub(super) fn handle_noise_map_tasks(
    mut commands: Commands,
    mut noise_map_tasks: Query<&mut ComputeNoiseMap>,
) {
    for mut task in noise_map_tasks.iter_mut() {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            debug!("Terrain generation task complete");
            commands.append(&mut commands_queue);
        }
    }
}
