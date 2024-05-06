use bevy::prelude::*;
pub use components::*;
use materials::*;
use systems::*;

use crate::core::{CursorActive, GameStates, ToolMode};

mod components;
mod materials;
mod systems;

pub const BUILDING_COST: u32 = 5;
pub const BUILDING_RADIUS: u32 = 16;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ValidBuildingToolMaterial>::default())
            .add_systems(OnEnter(GameStates::Playing), setup_building_tool)
            .add_systems(
                Update,
                (
                    follow_building_tool,
                    rotate_building_tool,
                    handle_building_tool.run_if(|cursor_active: Res<CursorActive>| **cursor_active),
                    check_building_tool_valid,
                    update_tool_ghost_material,
                )
                    .run_if(
                        in_state(GameStates::Playing).and_then(|tool_mode: Res<ToolMode>| {
                            matches!(*tool_mode, ToolMode::Build)
                        }),
                    ),
            )
            .add_systems(
                Update,
                (select_building_kind, update_ghost_building, building_increase_resource_count).run_if(in_state(GameStates::Playing)),
            );
    }
}
