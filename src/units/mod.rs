use std::{collections::VecDeque, f32::EPSILON};

use bevy::prelude::*;

use crate::{building::BuildingHasWorker, core::GameStates, quota::ResourceCount};

const CLOSE_ENOUGH: f32 = EPSILON;

#[derive(Component, Deref, DerefMut)]
pub struct UnitWaypoints(pub VecDeque<(Vec2, Vec<UnitWaypointAction>)>);

#[derive(Component, Deref)]
pub struct UnitVelocity(pub f32);

#[derive(Component)]
pub struct Unit;

#[derive(Component, Deref)]
struct UnitWaypointActions(pub Vec<UnitWaypointAction>);

#[derive(Clone, Debug, PartialEq)]
pub enum UnitWaypointAction {
    Gather(Entity),
    Deposit,
    Release(Entity),
}

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_unit_position, update_unit_waypoints, manage_unit_actions).run_if(in_state(GameStates::Playing)),
        );
    }
}

fn update_unit_position(
    mut q_units: Query<(&mut Transform, &UnitVelocity, &UnitWaypoints), With<Unit>>, time: Res<Time>,
) {
    for (mut transform, velocity, waypoints) in q_units.iter_mut() {
        if let Some((next_waypoint, _)) = waypoints.front() {
            let direction = *next_waypoint - transform.translation.xz();
            let distance = direction.length();
            let direction = direction.normalize();
            let delta = velocity.0 * time.delta_seconds();

            if distance < delta {
                transform.translation = next_waypoint.extend(transform.translation.y).xzy();
            } else {
                transform.translation += direction.extend(0.0).xzy() * delta;
            }
        }
    }
}

fn update_unit_waypoints(
    mut commands: Commands,
    mut q_units: Query<(Entity, &mut UnitWaypoints, &Transform), With<Unit>>
) {
    for (entity, mut waypoints, transform) in q_units.iter_mut() {
        if let Some((next_waypoint, actions)) = waypoints.front() {
            let direction = *next_waypoint - transform.translation.xz();
            let distance = direction.length();

            if distance < CLOSE_ENOUGH {
                commands.entity(entity).insert(UnitWaypointActions(actions.clone()));
                waypoints.pop_front();
            }
        }
    }
}

fn manage_unit_actions(
    mut commands: Commands,
    q_units: Query<(Entity, &UnitWaypointActions), With<Unit>>,
    mut resource_count: ResMut<ResourceCount>,
) {
    for (unit, actions) in q_units.iter() {
        commands.entity(unit).remove::<UnitWaypointActions>();

        for action in actions.iter() {
            match action {
                UnitWaypointAction::Gather(entity) => {
                    commands.entity(*entity).despawn_recursive();
                }
                UnitWaypointAction::Deposit => {
                    **resource_count += 1;
                }
                UnitWaypointAction::Release(entity) => {
                    commands.entity(*entity).remove::<BuildingHasWorker>();
                    commands.entity(unit).despawn_recursive();
                }
            }
        }
    }
}
