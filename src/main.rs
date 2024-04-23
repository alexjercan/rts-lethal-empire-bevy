use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    input::common_conditions::input_toggle_active,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use noise::utils::{ColorGradient, ImageRenderer, NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, MultiFractal, Perlin};

const CHUNK_SIZE: u32 = 32;
const CHUNK_SCALE: u32 = 32;

const BOUNDS_INTERVAL: f64 = 0.25;
const DISCOVER_SIZE: u32 = 2;
const DISCOVER_RENDER_SIZE: u32 = 2;

#[derive(Component)]
struct Chunk;

#[derive(Component)]
struct ChunkCoord(IVec2);

#[derive(Component)]
struct Ground;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(AssetCollection, Resource)]
struct GameAssets {}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::F1)))
        .add_plugins(PanOrbitCameraPlugin)
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .add_systems(OnEnter(GameStates::Playing), setup)
        .add_systems(Update, (draw_cursor,).run_if(in_state(GameStates::Playing)))
        .run();
}

fn map_generation(x: i32, y: i32) -> Image {
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

    let image = ImageRenderer::new()
        .set_gradient(ColorGradient::new().build_terrain_gradient())
        .render(&noise_map);

    let (width, height) = image.size();

    return Image::new(
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
}

fn discover(x: i32, y: i32, chunks: &Vec<ChunkCoord>) -> Vec<IVec2> {
    return (x - DISCOVER_SIZE as i32..x + DISCOVER_SIZE as i32)
        .into_iter()
        .flat_map(|x1| {
            (y - DISCOVER_SIZE as i32..y + DISCOVER_SIZE as i32)
                .into_iter()
                .map(move |y1| IVec2::new(x1, y1))
        })
        .filter(|coord| !chunks.iter().map(|c| c.0).any(|c| c == *coord))
        .collect();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // ground for clicks
    commands.spawn((GlobalTransform::default(), Ground));

    // plane
    let points = discover(0, 0, &Vec::new());
    points.iter()
        .map(|v| map_generation(v.x, v.y))
        .map(|img| images.add(img))
        .map(|handler| StandardMaterial::from(handler))
        .zip(points.iter())
        .for_each(|(material, point)| {
            commands.spawn((PbrBundle {
                mesh: meshes.add(
                    Plane3d::default()
                        .mesh()
                        .size(CHUNK_SIZE as f32, CHUNK_SIZE as f32),
                ),
                material: materials.add(material),
                transform: Transform::from_translation(point.extend(0).xzy().as_vec3() * CHUNK_SIZE as f32),
                ..default()
            },));
        });

    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings::NATURAL,
        PanOrbitCamera {
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
    ));
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let ground = ground_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the ground plane.
    let Some(distance) = ray.intersect_plane(ground.translation(), Plane3d::new(ground.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    // Draw a circle just above the ground plane at that position.
    gizmos.circle(
        point + ground.up() * 0.01,
        Direction3d::new_unchecked(ground.up()), // Up vector is already normalized.
        0.2,
        Color::WHITE,
    );
}
