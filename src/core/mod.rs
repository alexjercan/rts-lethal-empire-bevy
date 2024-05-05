use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use crate::{building::BuildingPlugin, terrain::TerrainPlugin};

#[cfg(feature = "debug")]
use crate::debug::DebugModePlugin;

pub use components::*;
pub use resources::*;
pub use states::*;
pub use assets::*;
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

        app.init_resource::<ToolMode>().add_systems(
            Update,
            select_tool_mode.run_if(in_state(GameStates::Playing)),
        );

        app.add_plugins(PanOrbitCameraPlugin)
            .add_plugins(TerrainPlugin::new(0))
            .add_plugins(BuildingPlugin)
            .init_state::<GameStates>()
            .add_loading_state(
                LoadingState::new(GameStates::AssetLoading)
                    .continue_to_state(GameStates::Playing)
                    .load_collection::<GameAssets>(),
            )
            .add_systems(OnEnter(GameStates::Playing), setup);
    }
}
