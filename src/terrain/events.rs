use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct DiscoverPositionEvent {
    pub(super) position: Vec2,
    pub(super) radius: u32,
}

impl DiscoverPositionEvent {
    pub fn new(position: Vec2, radius: u32) -> Self {
        DiscoverPositionEvent { position, radius }
    }
}
