use std::{collections::HashSet, f32::consts::FRAC_PI_2};

use bevy::prelude::*;

use crate::{
    assets::GameAssets,
    helpers,
    states::GameStates,
    terrain::{ChunkCoord, ChunkManager, TileCoord, TileKind, TileMapping},
    Obstacle, ToolMode,
};

#[derive(Component)]
pub struct BuildingTool;

#[derive(Component, Deref, DerefMut)]
pub struct BuildingToolValid(bool);

#[derive(Component)]
struct GhostBuilding;

#[derive(Component)]
struct Building;

#[derive(Component, Default, PartialEq, Eq, Clone, Hash, Debug)]
pub enum BuildingKind {
    #[default]
    LumberMill,
    StoneQuarry,
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameStates::Playing), setup_building_tool)
            .add_systems(
                Update,
                (
                    select_building_kind,
                    follow_building_tool,
                    rotate_building_tool,
                    handle_building_tool,
                    check_building_tool_valid,
                )
                    .run_if(in_state(GameStates::Playing).and_then(run_if_build_mode)),
            )
            .add_systems(
                Update,
                update_ghost_building.run_if(in_state(GameStates::Playing)),
            );
    }
}

fn run_if_build_mode(tool_mode: Res<ToolMode>) -> bool {
    matches!(*tool_mode, ToolMode::Build)
}

fn setup_building_tool(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn((
            BuildingTool,
            BuildingToolValid(false),
            BuildingKind::default(),
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(16.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            for (kind, scene) in &game_assets.buildings {
                parent.spawn((
                    GhostBuilding,
                    kind.clone(),
                    SceneBundle {
                        scene: scene.clone(),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                ));
            }
        });
}

fn update_ghost_building(
    tool_mode: Res<ToolMode>,
    q_tool: Query<(&BuildingKind, Ref<BuildingKind>), With<BuildingTool>>,
    mut q_ghost: Query<
        (&BuildingKind, &mut Visibility),
        (With<GhostBuilding>, Without<BuildingTool>),
    >,
) {
    let Ok((building_kind, component)) = q_tool.get_single() else {
        return;
    };

    if tool_mode.is_changed() || component.is_changed() {
        for (kind, mut visibility) in q_ghost.iter_mut() {
            *visibility = match *tool_mode {
                ToolMode::Build if *building_kind == *kind => Visibility::Visible,
                _ => Visibility::Hidden,
            }
        }
    }
}

fn select_building_kind(
    mut q_tool: Query<&mut BuildingKind, With<BuildingTool>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut building_kind) = q_tool.get_single_mut() else {
        return;
    };

    if input.just_pressed(KeyCode::Digit1) {
        *building_kind = BuildingKind::LumberMill;
    } else if input.just_pressed(KeyCode::Digit2) {
        *building_kind = BuildingKind::StoneQuarry;
    }
}

fn rotate_building_tool(
    mut q_tool: Query<&mut Transform, With<BuildingTool>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut building_tool_transform) = q_tool.get_single_mut() else {
        return;
    };

    if input.just_pressed(KeyCode::KeyR) {
        building_tool_transform.rotate(Quat::from_rotation_y(-FRAC_PI_2));
    }
}

fn follow_building_tool(
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_tool: Query<&mut Transform, With<BuildingTool>>,
    windows: Query<&Window>,
    chunk_manager: Res<ChunkManager>,
) {
    let Ok(mut building_tool_transform) = q_tool.get_single_mut() else {
        return;
    };
    let Ok((camera, camera_transform)) = q_camera.get_single() else {
        return;
    };
    let Some(point) = screen_to_world(camera, camera_transform, windows.single()) else {
        return;
    };

    let size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();
    let tile_pos = helpers::geometry::snap_to_tile(&point.xz(), &size, &tile_size);

    building_tool_transform.translation = tile_pos.extend(0.0).xzy();
}

fn handle_building_tool(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_tool: Query<(&BuildingKind, &BuildingToolValid, &Transform), With<BuildingTool>>,
    chunk_manager: Res<ChunkManager>,
    game_assets: Res<GameAssets>,
) {
    let Ok((building_kind, building_valid, tool_transform)) = q_tool.get_single() else {
        return;
    };

    if !**building_valid {
        return;
    }

    let point = tool_transform.translation;

    let size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();
    let chunk_coord = helpers::geometry::world_pos_to_chunk_coord(&point.xz(), &size, &tile_size);
    let Some(chunk) = chunk_manager.get(&chunk_coord) else {
        return;
    };

    let tile_coord = helpers::geometry::world_pos_to_tile_coord(&point.xz(), &size, &tile_size);
    let tile_pos = helpers::geometry::tile_coord_to_world_off(&tile_coord, &size, &tile_size);
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let scene = game_assets.buildings[&*building_kind].clone();

        commands.entity(*chunk).with_children(|parent| {
            parent.spawn((
                Building,
                TileCoord(tile_coord),
                Obstacle,
                SceneBundle {
                    scene,
                    transform: tool_transform.with_translation(tile_pos.extend(0.0).xzy()),
                    ..default()
                },
            ));
        });
    }
}

fn check_building_tool_valid(
    chunk_manager: Res<ChunkManager>,
    q_chunks: Query<(&Children, &TileMapping), With<ChunkCoord>>,
    q_tiles: Query<&TileCoord, With<Obstacle>>,
    mut q_tool: Query<(&mut BuildingToolValid, &Transform), With<BuildingTool>>,
) {
    let Ok((mut building_valid, tool_transform)) = q_tool.get_single_mut() else {
        return;
    };
    let point = tool_transform.translation;

    let size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();
    let chunk_coord = helpers::geometry::world_pos_to_chunk_coord(&point.xz(), &size, &tile_size);
    let tile_coord = helpers::geometry::world_pos_to_tile_coord(&point.xz(), &size, &tile_size);

    println!("{:?} in {:?}", tile_coord, chunk_coord);

    let Some(chunk) = chunk_manager.get(&chunk_coord) else {
        return;
    };
    let Ok((children, mapping)) = q_chunks.get(*chunk) else {
        return;
    };

    let obstacles = children
        .into_iter()
        .filter_map(|child| q_tiles.get(*child).ok().map(|x| **x))
        .collect::<HashSet<UVec2>>();

    let index = helpers::geometry::tile_coord_to_index(&tile_coord, &size);
    let tile_kind = mapping[index];

    **building_valid = !obstacles.contains(&tile_coord) && !matches!(tile_kind, TileKind::Water);
}

fn screen_to_world(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
) -> Option<Vec3> {
    let cursor_position = window.cursor_position()?;
    let ray = camera.viewport_to_world(camera_transform, cursor_position)?;
    let distance = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y))?;
    let point = ray.get_point(distance);

    Some(point)
}
