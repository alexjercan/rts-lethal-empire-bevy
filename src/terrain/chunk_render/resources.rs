use bevy::ecs::system::Resource;

#[derive(Debug, Clone, Copy, Resource)]
pub(super) struct ChunkRenderCPUConfig {}

impl Default for ChunkRenderCPUConfig {
    fn default() -> Self {
        ChunkRenderCPUConfig {}
    }
}
