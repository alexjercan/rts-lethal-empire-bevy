use bevy::prelude::*;
pub use components::*;
use materials::*;
use systems::*;

use crate::core::{GameStates, ToolMode};

mod materials;
mod components;
mod systems;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ValidBuildingToolMaterial>::default())
            .add_systems(OnEnter(GameStates::Playing), setup_building_tool)
            .add_systems(
                Update,
                (
                    select_building_kind,
                    follow_building_tool,
                    rotate_building_tool,
                    handle_building_tool,
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
                update_ghost_building.run_if(in_state(GameStates::Playing)),
            );
    }
}
