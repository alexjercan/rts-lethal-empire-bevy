use bevy::prelude::*;

pub fn screen_to_world(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
) -> Option<Vec3> {
    let cursor_position = window.cursor_position()?;
    let ray = camera.viewport_to_world(camera_transform, cursor_position)?;
    let distance = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y))?;
    let point = ray.get_point(distance);

    Some(point)
}
