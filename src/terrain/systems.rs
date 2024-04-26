use bevy::prelude::*;
use itertools::Itertools;

use super::{components::{Chunk, ChunkCoord}, events::DiscoverPositionEvent, resources::TerrainConfig};

pub(super) fn discover_position(
    mut commands: Commands,
    mut ev_disvover_position: EventReader<DiscoverPositionEvent>,
    q_chunk_coors: Query<&ChunkCoord, With<Chunk>>,
    config: Res<TerrainConfig>,
) {
    let chunk_coords = q_chunk_coors.iter().map(|c| c.0).collect::<Vec<_>>();

    ev_disvover_position
        .read()
        .flat_map(|ev| {
            discover(
                (ev.position / config.chunk_size).as_ivec2(),
                ev.radius,
                &chunk_coords,
            )
        })
        .for_each(|p| {
            commands.spawn((Chunk, ChunkCoord(p)));
        });
}

fn discover(position: IVec2, radius: u32, chunks: &Vec<IVec2>) -> Vec<IVec2> {
    debug!(
        "Triggered discover for at ({:?}) with radius {}",
        position, radius
    );

    return (position.x - radius as i32..=position.x + radius as i32)
        .cartesian_product(position.y - radius as i32..=position.y + radius as i32)
        .map(|(x, y)| IVec2::new(x, y))
        .filter(|coord| !chunks.iter().any(|c| c == coord))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover() {
        let chunks = vec![IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(0, 1)];

        let result = discover(IVec2::new(0, 0), 1, &chunks);

        assert_eq!(result, vec![IVec2::new(-1, -1), IVec2::new(-1, 0), IVec2::new(-1, 1), IVec2::new(0, -1), IVec2::new(1, -1), IVec2::new(1, 1)]);
    }
}
