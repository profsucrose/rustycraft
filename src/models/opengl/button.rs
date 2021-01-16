use cgmath::Vector3;

use super::{tex_quad::TexQuad, text_renderer::TextRenderer};

pub struct Button {
    texquad: TexQuad,
    text: String,
    left_x: f32,
    right_x: f32,
    bottom_y: f32,
    top_y: f32
}

impl Button {
    pub unsafe fn new(text: &str, x: f32, y: f32, width: f32, height: f32, screen_width: u32, screen_height: u32) -> Button {
        let texquad = TexQuad::new("assets/textures/button_wide.png", gl::TEXTURE0, true, screen_width, screen_height);
        let left_x = x - width / 2.0;
        let right_x = left_x + width;
        let bottom_y = y - height / 2.0;
        let top_y = bottom_y + height;
        Button { texquad, left_x, right_x, bottom_y, top_y, text: text.to_string()  }
    }

    pub fn is_hovered(&self, mouse_x: f32, mouse_y: f32) -> bool {
        mouse_x > self.left_x && mouse_x < self.right_x && mouse_y > self.bottom_y && mouse_y < self.top_y
    }

    pub unsafe fn draw(&self, text_renderer: &TextRenderer, mouse_x: f32, mouse_y: f32) {
        let text_x = self.left_x + ((self.right_x - self.left_x) / 2.0);
        let mut text_height = 0.0;
        for c in self.text.as_bytes() {
            let ch = text_renderer.get_char(*c as char);
            let height = ch.bearing.y as f32;
            if height > text_height {
                text_height = height;
            }
        }
        let text_y = self.bottom_y + ((self.top_y - self.bottom_y) / 2.0) - text_height / 2.0;

        // draw text
        text_renderer.render_text_centered(self.text.as_str(), text_x, text_y, 1.0, Vector3::new(0.0, 0.0, 0.0));
        
        // text shadow
        //text_renderer.render_text_centered(self.text.as_str(), text_x + 1.0, text_y - 1.0, 1.0, Vector3::new(165.0 / 255.0, 154.0 / 255.0, 154.0 / 255.0));

        let opacity = if self.is_hovered(mouse_x, mouse_y) {
            0.8
        } else {
            1.0
        };
        self.texquad.draw(self.left_x, self.bottom_y, self.right_x, self.top_y, opacity);
    }
}