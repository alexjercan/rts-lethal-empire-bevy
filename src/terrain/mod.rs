use bevy::prelude::*;

use self::{chunk_compute::ChunkComputeCPUPlugin, chunk_render::ChunkRenderCPUPlugin, resources::TerrainConfig, systems::discover_position};
pub(crate) use self::events::DiscoverPositionEvent;

mod chunk_compute;
mod chunk_render;
mod systems;
mod events;
mod components;
mod resources;

#[derive(Debug, Default)]
pub struct TerrainPlugin {
    config: TerrainConfig,
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkComputeCPUPlugin::default());
        app.add_plugins(ChunkRenderCPUPlugin::default());
        app.add_event::<DiscoverPositionEvent>();
        app.insert_resource(self.config);
        app.add_systems(Update, discover_position);
    }
}
