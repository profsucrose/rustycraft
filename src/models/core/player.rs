use cgmath::Vector3;

use crate::models::opengl::camera::Camera;

use super::world::World;

const GRAVITY: f32 = -0.005;
const TERMINAL_VEL: f32 = -0.2;

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
        let position = self.camera.position - Vector3::<f32>::new(0.0, 1.0, 0.0) + self.camera.front / 4.0;
        let x = position.x.round() as i32;
        let y = position.y.round() as i32;
        let z = position.z.round() as i32; 
        if !world.air_at(x, y, z) {
            self.camera.position = old_position;
        }
    }

    pub fn update_alt(&mut self, world: &World) {
        let x = self.camera.position.x.round() as i32;
        let y = self.camera.position.y.round() as i32;
        let z = self.camera.position.z.round() as i32;
        let ground_y = world.highest_in_column_from_y(x, y, z).unwrap() + 2;
        self.camera.position.y += self.velocity_y;
        if self.camera.position.y < (ground_y as f32) - 0.001 {
            self.is_jumping = false;
            self.camera.position.y = ground_y as f32;
        }
        self.velocity_y += GRAVITY;
        if self.velocity_y < TERMINAL_VEL {
            self.velocity_y = TERMINAL_VEL;
        }
    }

    pub fn jump(&mut self) {
        if !self.is_jumping {
            self.velocity_y = 0.2;
            self.is_jumping = true;
        }
    }
}