use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use noise::utils::{ColorGradient, ImageRenderer};

use crate::terrain::{
    chunk_compute::ChunkNoiseMap,
    components::{Chunk, ChunkCoord},
    TerrainConfig,
};

use super::components::ChunkNoiseMapImage;

pub(super) fn handle_image_render(
    mut commands: Commands,
    q_chunks: Query<
        (Entity, &ChunkCoord, &ChunkNoiseMap),
        (With<Chunk>, Without<ChunkNoiseMapImage>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    config: Res<TerrainConfig>,
) {
    for (entity, chunk, noisemap) in q_chunks.iter() {
        let point = chunk.0;

        debug!("Rendering image for chunk {:?}", point);

        let image = ImageRenderer::new()
            .set_gradient(ColorGradient::new().build_terrain_gradient())
            .render(&noisemap.0);

        let (width, height) = image.size();

        let image = Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                ..default()
            },
            TextureDimension::D2,
            image.into_iter().flatten().collect(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );

        let handler = images.add(image);
        let material = StandardMaterial::from(handler);

        commands
            .entity(entity)
            .insert((PbrBundle {
                mesh: meshes.add(
                    Plane3d::default()
                        .mesh()
                        .size(config.chunk_size, config.chunk_size),
                ),
                material: materials.add(material),
                transform: Transform::from_translation(
                    point.extend(0).xzy().as_vec3() * config.chunk_size,
                ),
                ..default()
            },))
            .insert(ChunkNoiseMapImage);
    }
}
