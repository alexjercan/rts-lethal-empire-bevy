use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_TILE_SIZE: f32 = 16.0;

#[derive(Debug, Resource)]
pub struct ChunkManager {
    size: UVec2,
    tile_size: Vec2,
    chunks: HashMap<IVec2, Entity>,
    loaded: HashSet<IVec2>,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            size: UVec2::splat(CHUNK_SIZE as u32),
            tile_size: Vec2::splat(CHUNK_TILE_SIZE),
            chunks: HashMap::new(),
            loaded: HashSet::new(),
        }
    }
}

impl ChunkManager {
    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn tile_size(&self) -> Vec2 {
        self.tile_size
    }

    pub fn load(&mut self, coord: IVec2) {
        self.loaded.insert(coord);
    }

    pub fn unload(&mut self, coord: IVec2) {
        self.loaded.remove(&coord);
    }

    pub fn out_range(&self, coord: &IVec2, radius: i32) -> Vec<IVec2> {
        self.loaded
            .iter()
            .filter(|c| (*coord - **c).abs().max_element() > radius)
            .cloned()
            .collect()
    }

    pub fn insert(&mut self, coord: IVec2, entity: Entity) {
        self.chunks.insert(coord, entity);
    }

    pub fn get(&self, coord: &IVec2) -> Option<&Entity> {
        self.chunks.get(coord)
    }

    pub fn contains(&self, coord: &IVec2) -> bool {
        self.chunks.contains_key(coord)
    }

    pub fn loaded(&self, coord: &IVec2) -> bool {
        self.loaded.contains(coord)
    }
}
