use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use noise::utils::{ColorGradient, ImageRenderer};

use crate::{terrain::{
    components::{Chunk, ChunkCoord, ChunkData}, TerrainConfig
}, GameAssets};

use super::{components::ChunkNoiseMapImage, resources::ChunkRenderCPUConfig};

pub(super) fn handle_image_render(
    mut commands: Commands,
    q_chunks: Query<
        (Entity, &ChunkCoord, &ChunkData),
        (With<Chunk>, Without<ChunkNoiseMapImage>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    terrain_config: Res<TerrainConfig>,
    chunk_config: Res<ChunkRenderCPUConfig>,
    game_assets: Res<GameAssets>,
) {
    for (entity, chunk, data) in q_chunks.iter() {
        let point = chunk.0;

        debug!("Rendering image for chunk {:?}", point);

        let image = ImageRenderer::new()
            .set_gradient(ColorGradient::new().build_terrain_gradient())
            .render(&(&(data.terrain)).into());

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
                        .size(terrain_config.chunk_size, terrain_config.chunk_size),
                ),
                material: materials.add(material),
                transform: Transform::from_translation(
                    point.extend(0).xzy().as_vec3() * terrain_config.chunk_size,
                ),
                ..default()
            },))
            .insert(ChunkNoiseMapImage);

        for tree in data.forest.iter() {
            let point = tree.position;
            commands.spawn(SceneBundle {
                scene:
                if tree.noise < chunk_config.forest_threshold_noise {
                    game_assets.tree.clone()
                } else {
                    game_assets.tree_snow.clone()
                },
                transform: Transform::from_xyz(point.x, 0.0, point.y).with_scale(Vec3::splat(1.0)),
                ..Default::default()
            });
        }

        for rock in data.rocks.iter() {
            let point = rock.position;
            let rocks = &game_assets.rocks;
            let index = rand::random::<usize>() % rocks.len();

            commands.spawn(SceneBundle {
                scene: rocks[index].clone(),
                transform: Transform::from_xyz(point.x, 0.0, point.y).with_scale(Vec3::splat(5.0)),
                ..Default::default()
            });
        }
    }
}
