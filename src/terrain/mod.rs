use std::ops::Deref;

use bevy::{
    ecs::system::CommandQueue,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use itertools::Itertools;
use noise::utils::{ColorGradient, ImageRenderer, NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, MultiFractal, Perlin};

#[derive(Debug, Event)]
pub struct DiscoverPositionEvent {
    position: Vec2,
    radius: u32,
}

impl DiscoverPositionEvent {
    pub fn new(position: Vec2, radius: u32) -> Self {
        DiscoverPositionEvent { position, radius }
    }
}

#[derive(Component)]
struct Chunk;

#[derive(Component)]
struct ChunkCoord(IVec2);

#[derive(Debug, Clone, Copy, Resource)]
struct TerrainConfig {
    seed: u32,
    chunk_size: f32,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        TerrainConfig {
            seed: 0,
            chunk_size: 32.0,
        }
    }
}

#[derive(Debug, Default)]
pub struct TerrainPlugin {
    config: TerrainConfig,
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkComputeCPUPlugin::default());
        app.add_plugins(ChunkRenderCPUPlugin::default());
        app.add_event::<DiscoverPositionEvent>();
        app.insert_resource(self.config);
        app.add_systems(Update, discover_position);
    }
}

fn discover_position(
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

////

#[derive(Debug, Clone, Copy, Resource)]
struct ChunkComputeCPUConfig {
    frequency: f64,
    persistence: f64,
    lacunarity: f64,
    octaves: u32,
    scale: f32,
    bounds_interval: f64,
}

impl Default for ChunkComputeCPUConfig {
    fn default() -> Self {
        ChunkComputeCPUConfig {
            frequency: 1.0,
            persistence: 0.5,
            lacunarity: 2.0,
            octaves: 14,
            scale: 8.0,
            bounds_interval: 0.25,
        }
    }
}

#[derive(Debug, Default)]
struct ChunkComputeCPUPlugin {
    config: ChunkComputeCPUConfig,
}

impl Plugin for ChunkComputeCPUPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.add_systems(Update, (spawn_noise_map_tasks, handle_noise_map_tasks));
    }
}

#[derive(Component)]
struct ChunkNoiseMap(NoiseMap);

#[derive(Component)]
struct ComputeNoiseMap(Task<CommandQueue>);

fn spawn_noise_map_tasks(
    mut commands: Commands,
    q_chunk_entities: Query<
        (Entity, &ChunkCoord),
        (
            With<Chunk>,
            Without<ChunkNoiseMap>,
            Without<ComputeNoiseMap>,
        ),
    >,
    chunk_config: Res<ChunkComputeCPUConfig>,
    terrain_config: Res<TerrainConfig>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (entity, chunk) in q_chunk_entities.iter() {
        let point = chunk.0;
        let terrain_config = *terrain_config.deref();
        let chunk_config = *chunk_config.deref();

        let task = thread_pool.spawn(async move {
            let noise = noisemap(point, &terrain_config, &chunk_config);

            let mut command_queue = CommandQueue::default();

            command_queue.push(move |world: &mut World| {

                world
                    .entity_mut(entity)
                    .insert(ChunkNoiseMap(noise))
                    .remove::<ComputeNoiseMap>();
            });

            return command_queue;
        });

        debug!("Creating terrain generation task for chunk {:?}", point);

        commands.entity(entity).insert(ComputeNoiseMap(task));
    }
}

fn noisemap(coord: IVec2, terrain: &TerrainConfig, config: &ChunkComputeCPUConfig) -> NoiseMap {
    let fbm = Fbm::<Perlin>::new(terrain.seed)
        .set_frequency(config.frequency)
        .set_persistence(config.persistence)
        .set_lacunarity(config.lacunarity)
        .set_octaves(config.octaves as usize);

    let size = (terrain.chunk_size * config.scale) as usize;
    let noise_map = PlaneMapBuilder::new(fbm)
        .set_size(size, size)
        .set_x_bounds(
            (coord.x as f64) * config.bounds_interval - config.bounds_interval / 2.0,
            (coord.x as f64) * config.bounds_interval + config.bounds_interval / 2.0,
        )
        .set_y_bounds(
            (coord.y as f64) * config.bounds_interval - config.bounds_interval / 2.0,
            (coord.y as f64) * config.bounds_interval + config.bounds_interval / 2.0,
        )
        .build();

    return noise_map;
}

fn handle_noise_map_tasks(
    mut commands: Commands,
    mut noise_map_tasks: Query<&mut ComputeNoiseMap>,
) {
    for mut task in noise_map_tasks.iter_mut() {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            debug!("Terrain generation task complete");
            commands.append(&mut commands_queue);
        }
    }
}

/////

#[derive(Debug, Clone, Copy, Resource)]
struct ChunkRenderCPUConfig {}

impl Default for ChunkRenderCPUConfig {
    fn default() -> Self {
        ChunkRenderCPUConfig {}
    }
}

#[derive(Debug, Default)]
struct ChunkRenderCPUPlugin {
    config: ChunkRenderCPUConfig,
}

impl Plugin for ChunkRenderCPUPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
        app.add_systems(Update, handle_image_render);
    }
}

#[derive(Component)]
struct ChunkNoiseMapImage;

fn handle_image_render(
    mut commands: Commands,
    q_chunks: Query<(Entity, &ChunkCoord, &ChunkNoiseMap), (With<Chunk>, Without<ChunkNoiseMapImage>)>,
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
