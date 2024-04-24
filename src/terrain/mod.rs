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

pub struct TerrainPlugin;

#[derive(Debug, Event)]
pub struct DiscoverPositionEvent(pub Vec2);

#[derive(Component)]
struct Chunk;

#[derive(Component)]
struct ChunkCoord(IVec2);

#[derive(Component)]
struct ChunkNoiseMap(NoiseMap);

#[derive(Component)]
struct ComputeNoiseMap(Task<CommandQueue>);

#[derive(Component)]
struct ChunkNoiseMapImageTMP(Image);

const CHUNK_SIZE: u32 = 32;
const CHUNK_SCALE: u32 = 8;

const BOUNDS_INTERVAL: f64 = 0.25;
const DISCOVER_SIZE: u32 = 4;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DiscoverPositionEvent>();
        app.add_systems(
            Update,
            (
                discover_position,
                spawn_noise_map_tasks,
                handle_noise_map_tasks,
                handle_image_tmp,
            ),
        );
    }
}

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
) {
    let thread_pool = AsyncComputeTaskPool::get();
    for (entity, chunk) in q_chunk_entities.iter() {
        let point = chunk.0;

        let task = thread_pool.spawn(async move {
            let noise = noisemap(point.x, point.y);

            let mut command_queue = CommandQueue::default();

            command_queue.push(move |world: &mut World| {
                let image = ImageRenderer::new()
                    .set_gradient(ColorGradient::new().build_terrain_gradient())
                    .render(&noise);

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

                world
                    .entity_mut(entity)
                    .insert(ChunkNoiseMap(noise))
                    .insert(ChunkNoiseMapImageTMP(image))
                    .remove::<ComputeNoiseMap>();
            });

            return command_queue;
        });

        debug!("Creating terrain generation task for chunk {:?}", point);

        commands.entity(entity).insert(ComputeNoiseMap(task));
    }
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

fn handle_image_tmp(
    mut commands: Commands,
    q_chunk_entities: Query<(Entity, &ChunkCoord, &ChunkNoiseMapImageTMP), With<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, chunk, image) in q_chunk_entities.iter() {
        let point = chunk.0;
        let image = image.0.clone();

        let handler = images.add(image);
        let material = StandardMaterial::from(handler);

        commands
            .entity(entity)
            .insert((PbrBundle {
                mesh: meshes.add(
                    Plane3d::default()
                        .mesh()
                        .size(CHUNK_SIZE as f32, CHUNK_SIZE as f32),
                ),
                material: materials.add(material),
                transform: Transform::from_translation(
                    point.extend(0).xzy().as_vec3() * CHUNK_SIZE as f32,
                ),
                ..default()
            },))
            .remove::<ChunkNoiseMapImageTMP>();
    }
}

fn discover_position(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut ev_disvover_position: EventReader<DiscoverPositionEvent>,
    q_chunk_coors: Query<&ChunkCoord, With<Chunk>>,
) {
    let chunk_coords = q_chunk_coors.iter().map(|c| c.0).collect::<Vec<_>>();
    let discover_chunks = ev_disvover_position
        .read()
        .map(|ev| position_to_chunk(ev.0))
        .flat_map(|p| discover(p.x, p.y, &chunk_coords));

    discover_chunks.for_each(|p| {
        debug!("Triggered discover for new chunk {:?}", p);

        let material = StandardMaterial::default();

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    Plane3d::default()
                        .mesh()
                        .size(CHUNK_SIZE as f32, CHUNK_SIZE as f32),
                ),
                material: materials.add(material),
                transform: Transform::from_translation(
                    p.extend(0).xzy().as_vec3() * CHUNK_SIZE as f32,
                ),
                ..default()
            },
            Chunk,
            ChunkCoord(p),
        ));
    });
}

fn position_to_chunk(position: Vec2) -> IVec2 {
    IVec2::new(
        (position.x / CHUNK_SIZE as f32) as i32,
        (position.y / CHUNK_SIZE as f32) as i32,
    )
}

fn noisemap(x: i32, y: i32) -> NoiseMap {
    let fbm = Fbm::<Perlin>::new(0)
        .set_frequency(1.0)
        .set_persistence(0.5)
        .set_lacunarity(2.0)
        .set_octaves(14);

    let noise_map = PlaneMapBuilder::new(fbm)
        .set_size(
            (CHUNK_SIZE * CHUNK_SCALE) as usize,
            (CHUNK_SIZE * CHUNK_SCALE) as usize,
        )
        .set_x_bounds(
            (x as f64) * BOUNDS_INTERVAL - BOUNDS_INTERVAL / 2.0,
            (x as f64) * BOUNDS_INTERVAL + BOUNDS_INTERVAL / 2.0,
        )
        .set_y_bounds(
            (y as f64) * BOUNDS_INTERVAL - BOUNDS_INTERVAL / 2.0,
            (y as f64) * BOUNDS_INTERVAL + BOUNDS_INTERVAL / 2.0,
        )
        .build();

    /*
        self.clear_gradient()
            .add_gradient_point(-1.00,              [  0,   0,   0, 255])
            .add_gradient_point(-256.0 / 16384.0,   [  6,  58, 127, 255])
            .add_gradient_point(-1.0 / 16384.0,     [ 14, 112, 192, 255])
            .add_gradient_point(0.0,                [ 70, 120,  60, 255])
            .add_gradient_point(1024.0 / 16384.0,   [110, 140,  75, 255])
            .add_gradient_point(2048.0 / 16384.0,   [160, 140, 111, 255])
            .add_gradient_point(3072.0 / 16384.0,   [184, 163, 141, 255])
            .add_gradient_point(4096.0 / 16384.0,   [128, 128, 128, 255])
            .add_gradient_point(5632.0 / 16384.0,   [128, 128, 128, 255])
            .add_gradient_point(6144.0 / 16384.0,   [250, 250, 250, 255])
            .add_gradient_point(1.0,                [255, 255, 255, 255])
    */

    return noise_map;
}

fn discover(x: i32, y: i32, chunks: &Vec<IVec2>) -> Vec<IVec2> {
    return (x - DISCOVER_SIZE as i32..=x + DISCOVER_SIZE as i32)
        .cartesian_product(y - DISCOVER_SIZE as i32..=y + DISCOVER_SIZE as i32)
        .map(|(x, y)| IVec2::new(x, y))
        .filter(|coord| !chunks.iter().any(|c| c == coord))
        .collect();
}
