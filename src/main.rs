mod vertex;
use vertex::Vertex;

use std::{io::Cursor, time::{Duration, Instant}};
use glium::{Display, Program, Surface, VertexBuffer, glutin::{ContextBuilder, event, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder}, index::{NoIndices, PrimitiveType}, texture::{self, RawImage2d}, uniform, uniforms::EmptyUniforms};

fn main() {
    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        uniform mat4 matrix;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;

        uniform sampler2D tex;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop).unwrap();

    // compile shaders
    let indices = NoIndices(PrimitiveType::TrianglesList);
    let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    // triangle vertices
    let vertex1 = Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0] };
    let vertex2 = Vertex { position: [ 0.0,  0.5], tex_coords: [0.0, 1.0] };
    let vertex3 = Vertex { position: [ 0.5, -0.25], tex_coords: [1.0, 0.0] };
    let shape = vec![vertex1, vertex2, vertex3];

    // create buffer
    let vertex_buffer = VertexBuffer::new(&display, &shape).unwrap();
    
    // load image
    let image = image::load(
        Cursor::new(&include_bytes!("../assets/textures/grass.png")[..]), 
        image::ImageFormat::Png
    ).unwrap().to_rgb8();

    let image_dimensions = image.dimensions();
    let image = RawImage2d::from_raw_rgb(image.into_raw(), image_dimensions);

    println!("{:?} {} {}", image_dimensions, image.height, image.width);
    let texture = glium::texture::texture2d::Texture2d::new(&display, image).unwrap();

    let mut t: f32 = -0.5;
    event_loop.run(move |ev, _, control_flow| {
        t += 0.0002;
        if t > 0.5 {
            t = -0.5;
        }

        // draw calls
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        // calculate uniform matrix
        let uniforms = uniform! {
            matrix: [
                [t.cos(), t.sin(), 0.0, 0.0],
                [-t.sin(), t.cos(), 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [t, 0.0, 0.0, 1.0],
            ],
            tex: &texture
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        // suspend thread for ~17ms for 60 FPS
        let next_frame_time = Instant::now(); // + Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        // close window on close event
        match ev {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                },
                _ => return
            },
            _ => ()
        }
    })
}
    