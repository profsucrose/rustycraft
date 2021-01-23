use cgmath::{Matrix4, Vector3};

use crate::models::{core::block_type::BlockType, opengl::{camera::Camera, cube::Cube}, traits::game_world::GameWorld};

use super::world::World;

const GRAVITY: f32 = -0.005;
const TERMINAL_VEL: f32 = -0.6;

pub struct Player {
    pub camera: Camera,
    is_jumping: bool,
    velocity_y: f32,
    model: Cube
}

impl Player {
    pub unsafe fn new(screen_width: u32, screen_height: u32) -> Player {
        let camera = Camera::new(screen_width, screen_height, 0.008);
        Player { camera, is_jumping: false, velocity_y: TERMINAL_VEL, model: Cube::new() }
    }

    pub unsafe fn draw_model(&mut self) {
        let position = self.camera.position;
        self.model.draw(
            &self.camera, 
            Matrix4::<f32>::from_translation(Vector3::new(position.x + 4.0, position.y - 0.51, position.z)) 
                * Matrix4::from_nonuniform_scale(1.0, 2.0, 1.0)
        );
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