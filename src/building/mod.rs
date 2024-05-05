use bevy::prelude::*;

use crate::{
    states::GameStates,
    terrain::{self, ChunkManager},
    ToolMode,
};

#[derive(Component)]
pub struct BuildingTool;

#[derive(Component, Deref)]
struct GhostBuilding(BuildingKind);

#[derive(Resource, Default, PartialEq, Eq)]
pub enum BuildingKind {
    #[default]
    LumberMill,
    StoneQuarry,
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuildingKind>()
            .add_systems(OnEnter(GameStates::Playing), setup_building_tool)
            .add_systems(
                Update,
                (
                    select_building_kind,
                    follow_building_tool,
                    handle_building_tool,
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

fn setup_building_tool(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            BuildingTool,
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                GhostBuilding(BuildingKind::LumberMill),
                meshes.add(Cuboid::new(16.0, 16.0, 16.0)),
                materials.add(Color::TOMATO),
                SpatialBundle {
                    transform: Transform::from_xyz(0.0, 8.0, 0.0),
                    visibility: Visibility::Hidden,
                    ..default()
                },
            ));

            parent.spawn((
                GhostBuilding(BuildingKind::StoneQuarry),
                meshes.add(Cuboid::new(16.0, 16.0, 16.0)),
                materials.add(Color::TURQUOISE),
                SpatialBundle {
                    transform: Transform::from_xyz(0.0, 8.0, 0.0),
                    visibility: Visibility::Hidden,
                    ..default()
                },
            ));
        });
}

fn update_ghost_building(
    mut q_ghost: Query<(&mut Visibility, &GhostBuilding)>,
    tool_mode: Res<ToolMode>,
    building_kind: Res<BuildingKind>,
) {
    if tool_mode.is_changed() || building_kind.is_changed() {
        for (mut visibility, kind) in q_ghost.iter_mut() {
            *visibility = match *tool_mode {
                ToolMode::Build if *building_kind == **kind => Visibility::Visible,
                _ => Visibility::Hidden,
            }
        }
    }
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

fn follow_building_tool(
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_building_tool: Query<&mut Transform, With<BuildingTool>>,
    windows: Query<&Window>,
    chunk_manager: Res<ChunkManager>,
) {
    let Ok((camera, camera_transform)) = q_camera.get_single() else {
        return;
    };
    let Ok(mut building_tool_transform) = q_building_tool.get_single_mut() else {
        return;
    };
    let Some(point) = screen_to_world(camera, camera_transform, windows.single()) else {
        return;
    };

    let size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();
    let tile_pos = terrain::helpers::geometry::snap_to_tile(&point.xz(), &size, &tile_size);

    building_tool_transform.translation = tile_pos.extend(0.0).xzy();
}

fn select_building_kind(mut building_kind: ResMut<BuildingKind>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Digit1) {
        *building_kind = BuildingKind::LumberMill;
    } else if input.just_pressed(KeyCode::Digit2) {
        *building_kind = BuildingKind::StoneQuarry;
    }
}

fn handle_building_tool(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    building_kind: Res<BuildingKind>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_manager: Res<ChunkManager>,
) {
    let Ok((camera, camera_transform)) = q_camera.get_single() else {
        return;
    };

    let Some(point) = screen_to_world(camera, camera_transform, windows.single()) else {
        return;
    };

    let size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();
    let chunk_coord =
        terrain::helpers::geometry::world_pos_to_chunk_coord(&point.xz(), &size, &tile_size);
    let Some(chunk) = chunk_manager.get(&chunk_coord) else {
        return;
    };

    let tile_coord =
        terrain::helpers::geometry::world_pos_to_tile_coord(&point.xz(), &size, &tile_size);
    let tile_pos =
        terrain::helpers::geometry::tile_coord_to_world_off(&tile_coord, &size, &tile_size);
    if mouse_button_input.just_pressed(MouseButton::Left) {
        match *building_kind {
            BuildingKind::LumberMill => {
                commands.entity(*chunk).with_children(|parent| {
                    parent.spawn((
                        meshes.add(Cuboid::new(16.0, 16.0, 16.0)),
                        materials.add(Color::TOMATO),
                        SpatialBundle {
                            transform: Transform::from_translation(tile_pos.extend(8.0).xzy()),
                            ..default()
                        },
                    ));
                });
            }
            BuildingKind::StoneQuarry => {
                commands.entity(*chunk).with_children(|parent| {
                    parent.spawn((
                        meshes.add(Cuboid::new(16.0, 16.0, 16.0)),
                        materials.add(Color::TURQUOISE),
                        SpatialBundle {
                            transform: Transform::from_translation(tile_pos.extend(8.0).xzy()),
                            ..default()
                        },
                    ));
                });
            }
        }
    }
}
