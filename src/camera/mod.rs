use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hack_panorbit_camera);
    }
}

// TODO: this is a hack. stop being a pussy and actually do a proper camera setup
pub fn hack_panorbit_camera(mut q_camera: Query<&mut PanOrbitCamera>) {
    for mut camera in q_camera.iter_mut() {
        camera.target_focus.y = 0.0;
    }
}
