use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    input::common_conditions::input_toggle_active,
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

mod terrain;

use terrain::{DiscoverPositionEvent, ResourcePiece, TerrainPlugin};

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct ResourceMiner;

#[derive(Component)]
struct MineIntervalTimer(Timer);

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(Default)]
enum BuildBrushType {
    #[default]
    Discover,
    PlaceMine,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct BuildBrush(BuildBrushType);

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "models/lowpoly_tree/tree.gltf#Scene0")]
    tree: Handle<Scene>,
    #[asset(path = "models/lowpoly_tree/tree_snow.gltf#Scene0")]
    tree_snow: Handle<Scene>,
    #[asset(
        paths(
            "models/lowpoly_stone/stone_tallA.glb#Scene0",
            "models/lowpoly_stone/stone_tallB.glb#Scene0",
            "models/lowpoly_stone/stone_tallC.glb#Scene0",
            "models/lowpoly_stone/stone_tallD.glb#Scene0",
            "models/lowpoly_stone/stone_tallE.glb#Scene0",
            "models/lowpoly_stone/stone_tallF.glb#Scene0",
            "models/lowpoly_stone/stone_tallG.glb#Scene0",
            "models/lowpoly_stone/stone_tallH.glb#Scene0",
            "models/lowpoly_stone/stone_tallI.glb#Scene0",
            "models/lowpoly_stone/stone_tallJ.glb#Scene0",
        ),
        collection(typed)
    )]
    rocks: Vec<Handle<Scene>>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            ..default()
        }))
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::F1)))
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(TerrainPlugin::default())
        .init_resource::<BuildBrush>()
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .add_systems(OnEnter(GameStates::Playing), setup)
        .add_systems(
            Update,
            (update_camera_focus, draw_cursor, pick_build_brush, mine_resource)
                .run_if(in_state(GameStates::Playing)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // ground for clicks
    commands.spawn((GlobalTransform::default(), Ground));

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

fn update_camera_focus(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    mut panorbit_camera_query: Query<&mut PanOrbitCamera>,
    windows: Query<&Window>,
) {
    let (camera, camera_transform) = camera_query.single();
    let ground = ground_query.single();

    let screen_center = Vec2::new(
        windows.single().width() / 2.0,
        windows.single().height() / 2.0,
    );

    let Some(ray) = camera.viewport_to_world(camera_transform, screen_center) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(ground.translation(), Plane3d::new(ground.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    let mut panorbit_camera = panorbit_camera_query.single_mut();
    panorbit_camera.target_focus = point.xz().extend(0.0).xzy();
    panorbit_camera.force_update = true;
}

fn draw_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
    mut ev_discover_position: EventWriter<DiscoverPositionEvent>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    build_brush: Res<BuildBrush>,
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

    match build_brush.0 {
        BuildBrushType::Discover => {
            if mouse_button_input.pressed(MouseButton::Left) {
                ev_discover_position.send(DiscoverPositionEvent::new(point.xz(), 4));
            }
        }
        BuildBrushType::PlaceMine => {
            if mouse_button_input.just_pressed(MouseButton::Left) {
                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Cylinder::new(0.25, 1.0)),
                        material: materials.add(Color::TURQUOISE),
                        transform: Transform::from_translation(point + Vec3::new(0.0, 0.5, 0.0)),
                        ..default()
                    })
                    .insert(ResourceMiner)
                    .insert(MineIntervalTimer(Timer::from_seconds(
                        0.5,
                        TimerMode::Repeating,
                    )));
            }
        }
    };
}

fn pick_build_brush(
    mut build_brush: ResMut<BuildBrush>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        build_brush.0 = BuildBrushType::Discover;
    }

    if keyboard_input.just_pressed(KeyCode::Digit2) {
        build_brush.0 = BuildBrushType::PlaceMine;
    }
}

const MINER_RANGE: f32 = 32.0;

fn mine_resource(
    mut gizmos: Gizmos,
    mut commands: Commands,
    q_resource_pieces: Query<(Entity, &Transform), With<ResourcePiece>>,
    mut q_resource_miners: Query<(&Transform, &mut MineIntervalTimer), (With<ResourceMiner>, Without<ResourcePiece>)>,
    time: Res<Time>,
) {
    for (miner_transform, mut miner_timer) in q_resource_miners.iter_mut() {
        gizmos
            .circle(miner_transform.translation.xz().extend(0.0).xzy(), Direction3d::Y, MINER_RANGE, Color::TURQUOISE)
            .segments(64);

        miner_timer.0.tick(time.delta());
        if miner_timer.0.finished() {
            if let Some((resource, resource_transform)) = q_resource_pieces
                .into_iter()
                .filter(|(_, resource_transform)| {
                    (resource_transform.translation.xz() - miner_transform.translation.xz()).length() < MINER_RANGE
                })
                .min_by(|(_, t1), (_, t2)| {
                    let d1 = (t1.translation.xz() - miner_transform.translation.xz()).length();
                    let d2 = (t2.translation.xz() - miner_transform.translation.xz()).length();
                    d1.total_cmp(&d2)
                })
            {
                gizmos.line(
                    miner_transform.translation,
                    resource_transform.translation,
                    Color::ORANGE_RED,
                );

                commands.entity(resource).despawn_recursive();
            }
        }
    }
}
