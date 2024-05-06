use bevy::prelude::*;

use crate::core::GameStates;

const QUOTA_TIME: f32 = 600.0;
const QUOTA_INITIAL: u32 = 10;
const RESOURCE_INITIAL: u32 = 5;

#[derive(Resource, Deref, DerefMut)]
pub struct QuotaTimer(pub Timer);

impl Default for QuotaTimer {
    fn default() -> Self {
        QuotaTimer(Timer::from_seconds(QUOTA_TIME, TimerMode::Repeating))
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Quota(pub u32);

impl Default for Quota {
    fn default() -> Self {
        Quota(QUOTA_INITIAL)
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ResourceCount(pub u32);

impl Default for ResourceCount {
    fn default() -> Self {
        ResourceCount(RESOURCE_INITIAL)
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct QuotaSuccess(pub bool);

pub struct QuotaPlugin;

impl Plugin for QuotaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuotaTimer>()
            .init_resource::<Quota>()
            .init_resource::<ResourceCount>()
            .init_resource::<QuotaSuccess>()
            .add_systems(Update, update_quota.run_if(in_state(GameStates::Playing)));
    }
}

fn update_quota(
    time: Res<Time>,
    mut timer: ResMut<QuotaTimer>,
    mut quota: ResMut<Quota>,
    mut resource_count: ResMut<ResourceCount>,
    mut quota_success: ResMut<QuotaSuccess>,
) {
    timer.tick(time.delta());
    if timer.finished() {
        if **resource_count < **quota {
            quota_success.0 = false;
        } else {
            **resource_count -= **quota;
            quota.0 *= 5;
            quota_success.0 = true;
        }
    }
}
