use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use crate::{building::BuildingPlugin, camera::CameraPlugin, quota::QuotaPlugin, terrain::TerrainPlugin, ui::UIPlugin};

#[cfg(feature = "debug")]
use crate::debug::DebugModePlugin;

pub use assets::*;
pub use components::*;
pub use resources::*;
pub use states::*;
use systems::*;

mod assets;
mod components;
mod resources;
mod states;
mod systems;

pub struct LethalEmpirePlugin;

impl Plugin for LethalEmpirePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.add_plugins(DebugModePlugin);

        #[cfg(not(feature = "debug"))]
        app.add_plugins(DefaultPlugins);

        #[cfg(feature = "debug")]
        app.add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::DEBUG,
            ..default()
        }));

        app.add_plugins(PanOrbitCameraPlugin)
            .add_plugins(CameraPlugin)
            .add_plugins(TerrainPlugin::new(0))
            .add_plugins(BuildingPlugin)
            .add_plugins(UIPlugin)
            .add_plugins(QuotaPlugin)
            .init_state::<GameStates>()
            .add_loading_state(
                LoadingState::new(GameStates::AssetLoading)
                    .continue_to_state(GameStates::Playing)
                    .load_collection::<GameAssets>(),
            )
            .init_resource::<ToolMode>()
            .init_resource::<CursorActive>()
            .add_systems(OnEnter(GameStates::Playing), setup)
            .add_systems(
                Update,
                clear_tool_mode.run_if(in_state(GameStates::Playing)),
            );
    }
}
