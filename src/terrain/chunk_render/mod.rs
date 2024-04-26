use bevy::prelude::*;

use self::{resources::ChunkRenderCPUConfig, systems::handle_image_render};

mod components;
mod resources;
mod systems;

#[derive(Debug, Default)]
pub(super) struct ChunkRenderCPUPlugin {
    config: ChunkRenderCPUConfig,
}

impl Plugin for ChunkRenderCPUPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.add_systems(Update, handle_image_render);
    }
}
