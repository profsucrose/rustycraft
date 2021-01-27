use cgmath::{Angle, Deg, Vector3, InnerSpace};

pub fn get_direction_from_mouse_move(sensitivity: f32, yaw: f32, pitch: f32, x_offset: f32, y_offset: f32) -> (f32, f32, Vector3<f32>) {
    let x_offset = x_offset * sensitivity;
    let y_offset = y_offset * sensitivity;

    let yaw = yaw + x_offset;
    let mut pitch = pitch + y_offset;

    // clamp pitch
    if pitch > 89.0 {
        pitch = 89.0;
    } else if pitch < -89.0 {
        pitch = -89.0;
    }

    let direction = Vector3::new(
        Deg(yaw).cos() * Deg(pitch).cos(),
        Deg(pitch).sin(),
        Deg(yaw).sin() * Deg(pitch).cos()
    );
    (yaw, pitch, direction.normalize())
}