use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub enum ToolMode {
    #[default]
    Select,
    Build,
}
