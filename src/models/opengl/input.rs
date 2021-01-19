use cgmath::Vector3;
use glfw::Key;

use super::{tex_quad::TexQuad, text_renderer::TextRenderer};

pub struct Input {
    texquad: TexQuad,
    pub text: String,
    left_x: f32,
    right_x: f32,
    bottom_y: f32,
    top_y: f32,
    focused: bool
}

impl Input {
    pub unsafe fn new(x: f32, y: f32, width: f32, height: f32, screen_width: u32, screen_height: u32) -> Input {
        let texquad = TexQuad::new("assets/textures/input.png", gl::TEXTURE0, true, screen_width, screen_height);
        let left_x = x - width / 2.0;
        let right_x = left_x + width;
        let bottom_y = y - height / 2.0;
        let top_y = bottom_y + height;
        Input { texquad, text: String::new(), left_x, right_x, bottom_y, top_y, focused: false }
    }

    pub fn update_focus(&mut self, mouse_x: f32, mouse_y: f32) {
        self.focused = mouse_x > self.left_x && mouse_x < self.right_x && mouse_y > self.bottom_y && mouse_y < self.top_y;
    }

    pub fn type_key(&mut self, key: Key, shift: bool) {
        if !self.focused {
            return;
        }

        println!("{:?}", key);
        if key == Key::Backspace {
            if self.text.len() > 0 {
                self.text.remove(self.text.len() - 1);
            }
            return;
        }

        if key as usize > Key::Z as usize {
            return;
        }

        let ch_id = (key as usize - Key::A as usize) + 65;
        println!("Char code decimal: {}", ch_id);
        let ch = if shift {
            // ; -> :
            let ch_id = if ch_id == 59 {
                58
            } else {
                ch_id
            };
            (ch_id as u8 as char).to_uppercase().nth(0).unwrap()
        } else {
            (ch_id as u8 as char).to_lowercase().nth(0).unwrap()
        };

        self.text.push_str(ch.to_string().as_str());
    }

    pub unsafe fn draw(&self, text_renderer: &TextRenderer) {
        let text_x = self.left_x + ((self.right_x - self.left_x) / 2.0);
        let text_height = text_renderer.get_char('d' as char).bearing.y as f32;
        let text_y = self.bottom_y + ((self.top_y - self.bottom_y) / 2.0) - text_height / 2.0;

        // draw text
        text_renderer.render_text_centered(self.text.as_str(), text_x, text_y, 1.0, Vector3::new(1.0, 1.0, 1.0));
        
        if self.focused {
            let width = text_renderer.calc_width(self.text.as_str(), 1.0);
            let mut underscore_x = text_x + width / 2.0;
            if self.text.len() > 0 {
                underscore_x += 1.0;
            }
            text_renderer.render_text("_", underscore_x, text_y, 1.0, Vector3::new(1.0, 1.0, 1.0));
        }

        // text shadow

        self.texquad.draw(self.left_x, self.bottom_y, self.right_x, self.top_y, 1.0);
    }
}