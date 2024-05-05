use bevy::prelude::*;
pub use components::*;
use materials::*;
use rand::{rngs::StdRng, RngCore, SeedableRng};
pub use resources::*;
use systems::*;

use crate::core::GameStates;

mod components;
mod materials;
mod resources;
mod systems;

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum TileKind {
    #[default]
    Water,
    Grass,
    Barren,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum ResourceKind {
    #[default]
    None,
    Tree,
    Rock,
}

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_TILE_SIZE: f32 = 16.0;
const SPAWN_CHUNK_RADIUS: usize = 8;
const LOAD_CHUNK_RADIUS: usize = 3;

pub struct TerrainPlugin {
    seed: u64,
}

impl TerrainPlugin {
    pub fn new(seed: u64) -> Self {
        TerrainPlugin { seed }
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        let mut seeder = StdRng::seed_from_u64(self.seed);

        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default())
            .init_resource::<ChunkManager>()
            .insert_resource(TerrainGenerator::new(seeder.next_u64()))
            .insert_resource(ResourceGenerator::new(seeder.next_u64()))
            .insert_resource(TerrainSeed(self.seed))
            .add_systems(
                Update,
                (
                    spawn_chunks_around_camera,
                    load_chunks_around_camera,
                    unload_chunks_outside_camera,
                    handle_chunks_tiles,
                    handle_chunks_resources,
                    generate_terrain_task,
                    handle_generate_terrain_task,
                    generate_resource_task,
                    handle_generate_resource_task,
                )
                    .run_if(in_state(GameStates::Playing)),
            );
    }
}
