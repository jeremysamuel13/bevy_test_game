use bevy::{input::Input, math::Vec3, prelude::*, render::camera::Camera};

// A simple camera system for moving and zooming the camera.
#[allow(dead_code)]
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.any_pressed([KeyCode::W, KeyCode::Up]) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.any_pressed([KeyCode::S, KeyCode::Down]) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        direction = direction.normalize_or_zero();

        let speed = if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            500.
        } else {
            250.
        };

        let z = transform.translation.z;
        transform.translation += time.delta_seconds() * direction * speed;
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}