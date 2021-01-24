use cgmath::{Matrix4, Vector3};

use crate::models::{core::block_type::BlockType, opengl::{camera::Camera, cube::Cube, face_uvs::FaceUVs, shader::Shader, texture::{self, Texture}}, traits::game_world::GameWorld};

use super::{face::Face, world::World};

const GRAVITY: f32 = -0.005;
const TERMINAL_VEL: f32 = -0.6;

pub struct Player {
    pub camera: Camera,
    is_jumping: bool,
    velocity_y: f32,
    head: Cube,
    torso: Cube,
    right_arm: Cube,
    left_arm: Cube,
    right_leg: Cube,
    left_leg: Cube
}

impl Player {
    pub unsafe fn new(screen_width: u32, screen_height: u32, texture_path: &str) -> Player {
        let texture = Texture::new(texture_path, gl::TEXTURE0, true);
        let head = Cube::new(
            texture,
            FaceUVs::new(0.0, 37.0, 10.0, 48.0, 48.0, 48.0),
            FaceUVs::new(10.0, 37.0, 20.0, 48.0, 48.0, 48.0),
            FaceUVs::new(20.0, 37.0, 30.0, 48.0, 48.0, 48.0),
            FaceUVs::new(30.0, 37.0, 40.0, 48.0, 48.0, 48.0),
            FaceUVs::new(0.0, 37.0, 10.0, 48.0, 48.0, 48.0),
            FaceUVs::new(0.0, 37.0, 10.0, 48.0, 48.0, 48.0)
        );
        let torso = Cube::new(
            texture,
            FaceUVs::new(0.0, 37.0, 11.0, 37.0, 48.0, 48.0),
            FaceUVs::new(11.0, 37.0, 22.0, 37.0, 48.0, 48.0),
            FaceUVs::new(22.0, 37.0, 33.0, 37.0, 48.0, 48.0),
            FaceUVs::new(33.0, 37.0, 44.0, 37.0, 48.0, 48.0),
            FaceUVs::new(0.0, 37.0, 11.0, 37.0, 48.0, 48.0),
            FaceUVs::new(0.0, 37.0, 11.0, 37.0, 48.0, 48.0)
        );
        let right_arm = Cube::new(
            texture,
            FaceUVs::new(0.0, 0.0, 6.0, 11.0, 48.0, 48.0),
            FaceUVs::new(6.0, 0.0, 12.0, 11.0, 48.0, 48.0),
            FaceUVs::new(12.0, 0.0, 18.0, 11.0, 48.0, 48.0),
            FaceUVs::new(18.0, 0.0, 24.0, 11.0, 48.0, 48.0),
            FaceUVs::new(24.0, 0.0, 30.0, 11.0, 48.0, 48.0),
            FaceUVs::new(30.0, 0.0, 36.0, 11.0, 48.0, 48.0),
        );
        let left_arm = Cube::new(
            texture,
            FaceUVs::new(0.0, 11.0, 6.0, 21.0, 48.0, 48.0),
            FaceUVs::new(6.0, 11.0, 12.0, 21.0, 48.0, 48.0),
            FaceUVs::new(12.0, 11.0, 18.0, 21.0, 48.0, 48.0),
            FaceUVs::new(18.0, 11.0, 24.0, 21.0, 48.0, 48.0),
            FaceUVs::new(24.0, 11.0, 30.0, 21.0, 48.0, 48.0),
            FaceUVs::new(30.0, 11.0, 36.0, 21.0, 48.0, 48.0),
        );
        let right_leg = Cube::new(
            texture,
            FaceUVs::new(0.0, 0.0, 6.0, 11.0, 48.0, 48.0),
            FaceUVs::new(6.0, 0.0, 12.0, 11.0, 48.0, 48.0),
            FaceUVs::new(12.0, 0.0, 18.0, 11.0, 48.0, 48.0),
            FaceUVs::new(18.0, 0.0, 24.0, 11.0, 48.0, 48.0),
            FaceUVs::new(24.0, 0.0, 30.0, 11.0, 48.0, 48.0),
            FaceUVs::new(30.0, 0.0, 36.0, 11.0, 48.0, 48.0),
        );
        let left_leg = Cube::new(
            texture,
            FaceUVs::new(0.0, 11.0, 6.0, 21.0, 48.0, 48.0),
            FaceUVs::new(6.0, 11.0, 12.0, 21.0, 48.0, 48.0),
            FaceUVs::new(12.0, 11.0, 18.0, 21.0, 48.0, 48.0),
            FaceUVs::new(18.0, 11.0, 24.0, 21.0, 48.0, 48.0),
            FaceUVs::new(24.0, 11.0, 30.0, 21.0, 48.0, 48.0),
            FaceUVs::new(30.0, 11.0, 36.0, 21.0, 48.0, 48.0),
        );
        let camera = Camera::new(screen_width, screen_height, 0.008);
        Player { camera, is_jumping: false, velocity_y: TERMINAL_VEL, head, torso, right_arm, left_arm, right_leg, left_leg }
    }

    pub unsafe fn draw_model(&mut self) {
        //let position = self.camera.position;

        self.head.draw(
            &self.camera, 
            Matrix4::from_translation(Vector3::new(0.0, 20.0, 0.0))
            * Matrix4::from_nonuniform_scale(0.7, 0.7, 0.7)
        );
        self.torso.draw(
            &self.camera, 
            Matrix4::from_translation(Vector3::new(0.0, 18.6, 0.0))
            * Matrix4::from_nonuniform_scale(1.3, 1.5, 0.8)
        );
        // self.right_arm.draw(
        //     &self.camera, 
        //     Matrix4::from_translation(Vector3::new(0.0, 19.0, 0.0))
        //     * Matrix4::from_nonuniform_scale(0.5, 2.0, 1.0)
        // );
        // self.left_arm.draw(
        //     &self.camera, 
        //     Matrix4::from_translation(Vector3::new(0.0, 20.0, 0.0))
        //     * Matrix4::from_nonuniform_scale(0.5, 2.0, 1.0)
        // );
        // self.right_leg.draw(
        //     &self.camera, 
        //     Matrix4::from_translation(Vector3::new(0.0, 20.0, 0.0))
        //     * Matrix4::from_nonuniform_scale(0.5, 2.0, 1.0)
        // );
        // self.left_leg.draw(
        //     &self.camera, 
        //     Matrix4::from_translation(Vector3::new(0.0, 20.0, 0.0))
        //     * Matrix4::from_nonuniform_scale(0.5, 2.0, 1.0)
        // );
    }

    pub fn update_position(&mut self, world: &impl GameWorld, deltatime: f32) {
        let old_position = self.camera.position.clone();
        self.camera.update_position(deltatime); 
        let sign = if self.camera.moving_backward { -1.0 } else { 1.0 };
        let position = self.camera.position - Vector3::<f32>::new(0.0, 1.0, 0.0) + sign * self.camera.front / 4.0;
        let x = position.x.round() as i32;
        let y = position.y.round() as i32;
        let z = position.z.round() as i32; 
        if !world.moveable(x, y, z) || !world.moveable(x, y + 1, z) {
            self.camera.position = old_position;
        }
    }

    pub fn update_alt(&mut self, world: &impl GameWorld) {
        let x = self.camera.position.x.round() as i32;
        let y = (self.camera.position.y + 0.04).round() as i32;
        let z = self.camera.position.z.round() as i32;
        let ground_y = world.highest_in_column_from_y(x, y, z);
        if ground_y.is_none() {
            return;
        }

        let ground_y = ground_y.unwrap() + 2;
        let test_y = self.camera.position.y + 0.1 + self.velocity_y as f32;
        if world.moveable(x, test_y.round() as i32, z) {
            self.camera.position.y = test_y as f32;
        } else {
            self.velocity_y = TERMINAL_VEL;
        }

        if self.camera.position.y < (ground_y as f32) - 0.001 {
            self.is_jumping = false;
            self.camera.position.y = ground_y as f32;
        }
        self.velocity_y += GRAVITY;
        if self.velocity_y < TERMINAL_VEL {
            self.velocity_y = TERMINAL_VEL;
        }
    }

    pub fn underwater(&self, world: &impl GameWorld) -> bool {
        let player_x = self.camera.position.x.round() as i32;
        let player_y = self.camera.position.y.round() as i32;
        let player_z = self.camera.position.z.round() as i32;
        match world.get_block(player_x, player_y, player_z) {
            Some(block) => block == BlockType::Water,
            None => false
        }
    }

    pub fn jump(&mut self) {
        if !self.is_jumping {
            self.velocity_y = 0.05;
            self.is_jumping = true;
        }
    }
}