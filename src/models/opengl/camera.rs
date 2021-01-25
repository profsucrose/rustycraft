use cgmath::{Angle, Deg, InnerSpace, Matrix4, Point3, Vector3, perspective, vec3};
use glfw::{Action, Key};

use crate::models::utils::vector_utils::get_direction_from_mouse_move;

#[derive(PartialEq, Clone, Copy)]
pub enum CameraMode {
    FirstPerson,
    Free
}

pub struct Camera {
    pub position: Vector3<f32>,
    pub front: Vector3<f32>,
    pub up: Vector3<f32>,
    pub moving_forward: bool,
    pub moving_backward: bool,
    pub moving_right: bool,
    pub moving_left: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub speed: f32
}

impl Camera {
    pub fn new(screen_width: u32, screen_height: u32, speed: f32) -> Camera {
        Camera {
            position: vec3(0.0, 0.0, 0.0),
            front: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
            moving_forward: false,
            moving_backward: false,
            moving_right: false,
            moving_left: false,
            pitch: 0.0,
            yaw: -90.0,
            fov: 45.0,
            screen_width,
            screen_height,
            speed
        }
    }

    pub fn process_keyboard(&mut self, key: Key, action: Action) {
        let pressed = match action {
            Action::Press => true,
            Action::Release => false,
            _ => return
        };
        match key {
            Key::W => self.moving_forward = pressed,
            Key::A => self.moving_left = pressed,
            Key::S => self.moving_backward = pressed,
            Key::D => self.moving_right = pressed,
            _ => ()
        }
    }

    pub fn update_position(&mut self, deltatime: f32, mode: CameraMode) {
        let speed = self.speed * deltatime;
        let old_y = self.position.y;

        let mut front = self.front;
        if mode == CameraMode::FirstPerson {
            front = Vector3::new(self.front.x, 0.0, self.front.z).normalize();
        }
        if self.moving_forward {
            self.position += speed * front
        } 
        
        if self.moving_backward {
            self.position -= speed * front
        } 
    
        if self.moving_right {
            self.position += speed * self.front.cross(self.up).normalize()
        } 

        if self.moving_left {
            self.position -= speed * self.front.cross(self.up).normalize()
        }

        if mode == CameraMode::FirstPerson {
            self.position.y = old_y;
        }
    }

    pub fn mouse_callback(&mut self, x_offset: f32, y_offset: f32) {
        let (yaw, pitch, direction) = get_direction_from_mouse_move(0.3, self.yaw, self.pitch, x_offset, y_offset);
        self.pitch = pitch;
        self.yaw = yaw;
        self.front = direction;
    }

    pub fn scroll_callback(&mut self, y_offset: f32) {
        self.fov -= y_offset;
        // clamp fov
        if self.fov < 1.0 {
            self.fov = 1.0
        } else if self.fov > 150.0 {
            self.fov = 150.0
        }
    }

    pub fn get_projection(&self) -> Matrix4<f32> {
        perspective(Deg(self.fov), (self.screen_width as f32) / (self.screen_height as f32), 0.01, 1000.0)
    }    

    pub fn get_view(&self) -> Matrix4<f32> {
        Matrix4::look_at(
            Point3::new(
                self.position.x, 
                self.position.y, 
                self.position.z
            ), 
            Point3::new(
                self.position.x + self.front.x, 
                self.position.y + self.front.y, 
                self.position.z + self.front.z
            ), 
            self.up
        )
    }
}