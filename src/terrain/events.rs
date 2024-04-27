use bevy::prelude::*;

#[derive(Debug, Event)]
pub(crate) struct DiscoverPositionEvent {
    pub(super) position: Vec2,
    pub(super) radius: u32,
}

impl DiscoverPositionEvent {
    pub(crate) fn new(position: Vec2, radius: u32) -> Self {
        DiscoverPositionEvent { position, radius }
    }
}
