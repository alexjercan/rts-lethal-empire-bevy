use bevy::{
    ecs::system::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};

use crate::states::GameStates;

use self::generator::{TerrainGenerator, TileKind};

use super::{chunking::ChunkManager, ChunkCoord};

mod generator;

#[derive(Component)]
struct ComputeTileMapping(Task<CommandQueue>);

#[derive(Component, Deref)]
pub struct TileMapping(Vec<TileKind>);

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainGenerator>().add_systems(
            Update,
            (generate_terrain_task, handle_generate_terrain_task)
                .run_if(in_state(GameStates::Playing)),
        );
    }
}

fn generate_terrain_task(
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

fn handle_generate_terrain_task(mut commands: Commands, mut tasks: Query<&mut ComputeTileMapping>) {
    for mut task in &mut tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
    }
}
