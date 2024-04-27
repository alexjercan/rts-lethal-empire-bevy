// Purpose: Chunk rendering system using CPU
// We want this plugin to spawn the graphics of our `Chunk` entities.

use bevy::prelude::*;
use noise::utils::NoiseMap;

use self::{resources::ChunkRenderCPUConfig, systems::handle_image_render};

use super::components::ChunkNoiseMap;

impl<'a> Into<NoiseMap> for &'a ChunkNoiseMap {
    fn into(self) -> NoiseMap {
        let mut noisemap = NoiseMap::new(self.size.x as usize, self.size.y as usize);
        noisemap.iter_mut().zip(self.map.iter()).for_each(|(a, &b)| *a = b);
        return noisemap.set_border_value(self.border_value);
    }
}

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
