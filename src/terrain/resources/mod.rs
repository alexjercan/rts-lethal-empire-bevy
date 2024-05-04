use bevy::{
    ecs::system::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};

use crate::states::GameStates;

use self::generator::ResourceGenerator;
pub use self::generator::ResourceKind;

use super::{chunking::ChunkManager, ChunkCoord};

mod generator;

#[derive(Component)]
struct ComputeResourceMapping(Task<CommandQueue>);

#[derive(Component, Deref)]
pub struct ResourceMapping(Vec<ResourceKind>);

pub struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceGenerator>().add_systems(
            Update,
            (generate_resource_task, handle_generate_resource_task)
                .run_if(in_state(GameStates::Playing)),
        );
    }
}

fn generate_resource_task(
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

fn handle_generate_resource_task(
    mut commands: Commands,
    mut tasks: Query<&mut ComputeResourceMapping>,
) {
    for mut task in &mut tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
    }
}
