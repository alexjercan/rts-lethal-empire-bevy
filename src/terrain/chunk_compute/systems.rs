use std::ops::Deref;

use bevy::{
    ecs::system::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool},
};
use noise::{utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder}, Fbm, MultiFractal, Perlin};

use crate::terrain::components::ChunkNoiseMap;

use super::{
    components::ComputeNoiseMap,
    resources::ChunkComputeCPUConfig,
    Chunk, ChunkCoord, TerrainConfig,
};

pub(super) fn spawn_noise_map_tasks(
    mut commands: Commands,
    q_chunk_entities: Query<
        (Entity, &ChunkCoord),
        (
            With<Chunk>,
            Without<ChunkNoiseMap>,
            Without<ComputeNoiseMap>,
        ),
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
            let noise = noisemap(point, &terrain_config, &chunk_config);

            let mut command_queue = CommandQueue::default();

            command_queue.push(move |world: &mut World| {
                world
                    .entity_mut(entity)
                    .insert(ChunkNoiseMap::from(noise))
                    .remove::<ComputeNoiseMap>();
            });

            return command_queue;
        });

        debug!("Creating terrain generation task for chunk {:?}", point);

        commands.entity(entity).insert(ComputeNoiseMap(task));
    }
}

fn noisemap(coord: IVec2, terrain: &TerrainConfig, config: &ChunkComputeCPUConfig) -> NoiseMap {
    let fbm = Fbm::<Perlin>::new(terrain.seed)
        .set_frequency(config.frequency)
        .set_persistence(config.persistence)
        .set_lacunarity(config.lacunarity)
        .set_octaves(config.octaves as usize);

    let size = (terrain.chunk_size * config.scale) as usize;
    let noise_map = PlaneMapBuilder::new(fbm)
        .set_size(size, size)
        .set_x_bounds(
            (coord.x as f64) * config.bounds_interval - config.bounds_interval / 2.0,
            (coord.x as f64) * config.bounds_interval + config.bounds_interval / 2.0,
        )
        .set_y_bounds(
            (coord.y as f64) * config.bounds_interval - config.bounds_interval / 2.0,
            (coord.y as f64) * config.bounds_interval + config.bounds_interval / 2.0,
        )
        .build();

    return noise_map;
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
