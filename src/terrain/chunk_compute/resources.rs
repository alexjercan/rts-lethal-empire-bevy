use bevy::ecs::system::Resource;

#[derive(Debug, Clone, Copy, Resource)]
pub(super) struct ChunkComputeCPUConfig {
    pub(super) frequency: f64,
    pub(super) persistence: f64,
    pub(super) lacunarity: f64,
    pub(super) octaves: u32,
    pub(super) scale: f32,
    pub(super) bounds_interval: f64,
}

impl Default for ChunkComputeCPUConfig {
    fn default() -> Self {
        ChunkComputeCPUConfig {
            frequency: 1.0,
            persistence: 0.5,
            lacunarity: 2.0,
            octaves: 14,
            scale: 8.0,
            bounds_interval: 0.125,
        }
    }
}
