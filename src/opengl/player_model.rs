use cgmath::{Deg, Matrix4, Vector3};

use super::{camera::Camera, cube::Cube, face_uvs::FaceUVs, texture::Texture};

pub struct PlayerModel {
    head: Cube,
    torso: Cube
}

impl PlayerModel {
    pub unsafe fn new(texture_path: &str) -> PlayerModel {
        let texture = Texture::new(texture_path, gl::TEXTURE0, true);
        let head = Cube::new(
            texture,
            FaceUVs::new(10.0, 10.0, 0.0, 0.0, 60.0, 20.0),
            FaceUVs::new(20.0, 10.0, 10.0, 0.0, 60.0, 20.0),
            FaceUVs::new(30.0, 10.0, 20.0, 0.0, 60.0, 20.0),
            FaceUVs::new(40.0, 10.0, 30.0, 0.0, 60.0, 20.0),
            FaceUVs::new(50.0, 10.0, 40.0, 0.0, 60.0, 20.0),
            FaceUVs::new(60.0, 10.0, 50.0, 0.0, 60.0, 20.0),
        );
        let torso = Cube::new(
            texture,
            FaceUVs::new(10.0, 20.0, 0.0, 10.0, 60.0, 20.0),
            FaceUVs::new(20.0, 20.0, 10.0, 10.0, 60.0, 20.0),
            FaceUVs::new(30.0, 20.0, 20.0, 10.0, 60.0, 20.0),
            FaceUVs::new(40.0, 20.0, 30.0, 10.0, 60.0, 20.0),
            FaceUVs::new(50.0, 20.0, 40.0, 10.0, 60.0, 20.0),
            FaceUVs::new(60.0, 20.0, 50.0, 10.0, 60.0, 20.0),
        );
        PlayerModel { head, torso }
    }

    pub unsafe fn draw(&self, camera: &Camera, position: Vector3<f32>, pitch: f32, yaw: f32) {
        let pitch_rotate = Matrix4::from_angle_x(Deg(pitch));
        let yaw_rotate = Matrix4::from_angle_y(Deg(-yaw - 90.0));
        self.head.draw(
            camera, 
            Matrix4::from_translation(Vector3::new(position.x, position.y - 0.35, position.z))
            * yaw_rotate
            * pitch_rotate
            * Matrix4::from_nonuniform_scale(0.7, 0.7, 0.7)
        );
        self.torso.draw(
            camera, 
            Matrix4::from_translation(Vector3::new(position.x, position.y - 1.1, position.z))
            * yaw_rotate 
            * Matrix4::from_nonuniform_scale(0.9, 0.7, 0.6)
        );
    }
}