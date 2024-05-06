use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub enum ToolMode {
    #[default]
    Select,
    Build,
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct CursorActive(pub bool);

impl Default for CursorActive {
    fn default() -> Self {
        CursorActive(true)
    }
}
