use cgmath::Vector3;

use crate::models::{core::block_type::BlockType, opengl::camera::Camera};

use super::world::World;

const GRAVITY: f32 = -0.005;
const TERMINAL_VEL: f32 = -0.6;

pub struct Player {
    pub camera: Camera,
    is_jumping: bool,
    velocity_y: f32
}

impl Player {
    pub fn new(screen_width: u32, screen_height: u32) -> Player {
        let camera = Camera::new(screen_width, screen_height, 0.008);
        Player { camera, is_jumping: false, velocity_y: TERMINAL_VEL }
    }

    pub fn update_position(&mut self, world: &World, deltatime: f32) {
        let old_position = self.camera.position.clone();
        self.camera.update_position(deltatime); 
        let sign = if self.camera.moving_backward { -1.0 } else { 1.0 };
        let position = self.camera.position - Vector3::<f32>::new(0.0, 1.0, 0.0) + sign * self.camera.front / 4.0;
        let x = position.x.round() as i32;
        let y = position.y.round() as i32;
        let z = position.z.round() as i32; 
        if !Player::moveable(world, x, y, z) || !Player::moveable(world, x, y + 1, z) {
            self.camera.position = old_position;
        }
    }

    pub fn update_alt(&mut self, world: &World) {
        let x = self.camera.position.x.round() as i32;
        let y = (self.camera.position.y + 0.04).round() as i32;
        let z = self.camera.position.z.round() as i32;
        let ground_y = world.highest_in_column_from_y(x, y, z).unwrap() + 2;

        let test_y = self.camera.position.y + 0.1 + self.velocity_y as f32;
        if Player::moveable(world, x, test_y.round() as i32, z) {
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

    pub fn underwater(&self, world: &World) -> bool {
        let player_x = self.camera.position.x.round() as i32;
        let player_y = self.camera.position.y.round() as i32;
        let player_z = self.camera.position.z.round() as i32;
        world.get_block(player_x, player_y, player_z).unwrap() == BlockType::Water
    }

    fn moveable(world: &World, world_x: i32, world_y: i32, world_z: i32) -> bool {
        let block = world.get_block(world_x, world_y, world_z).unwrap();
        block == BlockType::Air || block == BlockType::Water
    }

    pub fn jump(&mut self) {
        if !self.is_jumping {
            self.velocity_y = 0.05;
            self.is_jumping = true;
        }
    }
}