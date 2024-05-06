use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, utils::HashSet};

use crate::{
    core::{GameAssets, Obstacle, ToolMode},
    helpers,
    quota::ResourceCount,
    terrain::{ChunkCoord, ChunkManager, ResourceKind, TileCoord, TileKind, TileMapping},
};

use super::{
    Building, BuildingKind, BuildingTool, BuildingToolValid, BuildingValidGhost, GhostBuilding,
    ValidBuildingToolMaterial, BUILDING_COST, BUILDING_RADIUS,
};

pub fn setup_building_tool(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ValidBuildingToolMaterial>>,
) {
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
            parent.spawn((
                BuildingValidGhost,
                MaterialMeshBundle {
                    mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
                    material: materials.add(ValidBuildingToolMaterial::default()),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    visibility: Visibility::Hidden,
                    ..default()
                },
            ));

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

pub fn update_tool_ghost_material(
    q_tool: Query<&BuildingToolValid, With<BuildingTool>>,
    q_tool_ghost: Query<&Handle<ValidBuildingToolMaterial>, With<BuildingValidGhost>>,
    mut materials: ResMut<Assets<ValidBuildingToolMaterial>>,
) {
    let Ok(building_valid) = q_tool.get_single() else {
        return;
    };

    for handle in q_tool_ghost.iter() {
        if let Some(material) = materials.get_mut(handle) {
            material.valid = **building_valid as u32;
        }
    }
}

pub fn update_ghost_building(
    tool_mode: Res<ToolMode>,
    q_tool: Query<(&BuildingKind, Ref<BuildingKind>), With<BuildingTool>>,
    mut q_ghost: Query<
        (&BuildingKind, &mut Visibility),
        (With<GhostBuilding>, Without<BuildingTool>),
    >,
    mut q_tool_ghost: Query<&mut Visibility, (With<BuildingValidGhost>, Without<GhostBuilding>)>,
) {
    let Ok((building_kind, component)) = q_tool.get_single() else {
        return;
    };

    if tool_mode.is_changed() || component.is_changed() {
        for mut visibility in q_tool_ghost.iter_mut() {
            *visibility = match *tool_mode {
                ToolMode::Build => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }

        for (kind, mut visibility) in q_ghost.iter_mut() {
            *visibility = match *tool_mode {
                ToolMode::Build if *building_kind == *kind => Visibility::Visible,
                _ => Visibility::Hidden,
            }
        }
    }
}

pub fn select_building_kind(
    mut tool_mode: ResMut<ToolMode>,
    mut q_tool: Query<&mut BuildingKind, With<BuildingTool>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut building_kind) = q_tool.get_single_mut() else {
        return;
    };

    if input.just_pressed(KeyCode::Digit1) {
        *tool_mode = ToolMode::Build;
        *building_kind = BuildingKind::LumberMill;
    } else if input.just_pressed(KeyCode::Digit2) {
        *tool_mode = ToolMode::Build;
        *building_kind = BuildingKind::StoneQuarry;
    }
}

pub fn rotate_building_tool(
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

pub fn follow_building_tool(
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
    let Some(point) = helpers::camera::screen_to_world(camera, camera_transform, windows.single())
    else {
        return;
    };

    let size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();
    let tile_pos = helpers::geometry::snap_to_tile(&point.xz(), &size, &tile_size);

    building_tool_transform.translation = tile_pos.extend(0.0).xzy();
}

pub fn handle_building_tool(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_tool: Query<(&BuildingKind, &BuildingToolValid, &Transform), With<BuildingTool>>,
    chunk_manager: Res<ChunkManager>,
    game_assets: Res<GameAssets>,
    mut resources: ResMut<ResourceCount>,
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
        **resources -= BUILDING_COST;

        let scene = game_assets.buildings[&*building_kind].clone();

        commands.entity(*chunk).with_children(|parent| {
            parent.spawn((
                Building,
                building_kind.clone(),
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

pub fn check_building_tool_valid(
    chunk_manager: Res<ChunkManager>,
    q_chunks: Query<(&Children, &TileMapping), With<ChunkCoord>>,
    q_tiles: Query<&TileCoord, With<Obstacle>>,
    mut q_tool: Query<(&mut BuildingToolValid, &Transform), With<BuildingTool>>,
    resources: Res<ResourceCount>,
) {
    let Ok((mut building_valid, tool_transform)) = q_tool.get_single_mut() else {
        return;
    };
    let point = tool_transform.translation;

    let size = chunk_manager.size();
    let tile_size = chunk_manager.tile_size();
    let chunk_coord = helpers::geometry::world_pos_to_chunk_coord(&point.xz(), &size, &tile_size);
    let tile_coord = helpers::geometry::world_pos_to_tile_coord(&point.xz(), &size, &tile_size);

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

    let is_water = matches!(tile_kind, TileKind::Water);
    let is_blocked = obstacles.contains(&tile_coord);
    let has_resources = **resources >= BUILDING_COST;

    **building_valid = !is_blocked && !is_water && has_resources;
}

pub fn building_increase_resource_count(
    mut commands: Commands,
    chunk_manager: Res<ChunkManager>,
    mut resource_count: ResMut<ResourceCount>,
    q_buildings: Query<(&GlobalTransform, &BuildingKind), With<Building>>,
    q_chunks: Query<&Children, With<ChunkCoord>>,
    q_resources: Query<(Entity, &GlobalTransform, &ResourceKind)>,
) {
    for (building_transform, building_kind) in q_buildings.iter() {
        let point = building_transform.translation().xz();

        let size = chunk_manager.size();
        let tile_size = chunk_manager.tile_size();
        let chunk_coords = helpers::geometry::world_area_to_chunk_coords(
            &point,
            BUILDING_RADIUS,
            &size,
            &tile_size,
        );

        let chunks = chunk_coords
            .iter()
            .filter_map(|coord| chunk_manager.get(coord));

        if let Some((closest, _position, distance)) = chunks
            .filter_map(|chunk| q_chunks.get(*chunk).ok())
            .flatten()
            .filter_map(|child| q_resources.get(*child).ok())
            .filter(|(_, _, kind)| **kind == building_kind.clone().into())
            .map(|(entity, transform, _)| (entity, transform.translation().xz()))
            .map(|(entity, pos)| (entity, pos, pos.distance(point)))
            .min_by_key(|(_, _, dist)| *dist as i32)
        {
            if distance < BUILDING_RADIUS as f32 * tile_size.x.max(tile_size.y) {
                **resource_count += 1;
                commands.entity(closest).despawn_recursive();
            }
        }
    }
}
