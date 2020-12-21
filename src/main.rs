//mod vertex;
mod shader;
mod teapot;
mod view_matrix;

use view_matrix::view_matrix;
use event::VirtualKeyCode;
use shader::create_shader_program;
use std::{io::Cursor, time::{Duration, Instant}};
use glium::{BackfaceCullingMode, Display, Surface, VertexBuffer, glutin::{self, ContextBuilder, event::{self, ElementState}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder}, index::{NoIndices, PrimitiveType}, texture::{self, RawImage2d}, uniform, uniforms::EmptyUniforms};

fn main() {
    let event_loop = EventLoop::new();
    let mut wb = WindowBuilder::new();
    wb = wb.with_title("Hello World!");

    let cb = ContextBuilder::new();
    let display = Display::new(wb, cb, &event_loop).unwrap();

    // compile shaders
    let program = create_shader_program(&display, "/Users/moffice/Documents/Rust/glium-demo/assets/shaders/vertex.vert", "/Users/moffice/Documents/Rust/glium-demo/assets/shaders/fragment.frag");

    let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &teapot::INDICES).unwrap();

    // depth buffer
    let cb = ContextBuilder::new().with_depth_buffer(24);

    let mut moving_right = false;
    let mut x = 0f32;
    event_loop.run(move |ev, _, control_flow| {
        // draw calls
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        // perspective matrix
        let perspective = {
            let (width, height) = target.get_dimensions();
            let aspect_ratio = height as f32 / width as f32;

            let fov: f32 = 3.141592 / 3.0;
            let zfar = 1024.0;
            let znear = 0.1;

            let f = 1.0 / (fov / 2.0).tan();

            [
                [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
                [         0.0         ,     f ,              0.0              ,   0.0],
                [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
                [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
            ]
        };

        if moving_right {
            x += 0.01
        }
        let view = view_matrix(&[x, -1.0, 1.0], &[-2.0, 1.0, 1.0], &[0.0, 1.0, 0.0]);

        // calculate uniform matrix
        let uniforms = uniform! {
            model: [
                [0.01, 0.0, 0.0, 0.0],
                [0.0, 0.01, 0.0, 0.0],
                [0.0, 0.0, 0.01, 0.0],
                [0.0, 0.0, 2.0, 1.0f32]
            ],
            perspective: perspective,
            view: view,
            u_light: [1.4, 0.4, -0.7f32]
        };

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        
        target.draw((&positions, &normals), &indices, &program, &uniforms, &params).unwrap();
        target.finish().unwrap();

        // suspend thread for ~17ms for 60 FPS
        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        // close window on close event
        match ev {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _ } => {
                    if let Some(key) = input.virtual_keycode {
                        match key {
                            VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                            VirtualKeyCode::D => moving_right = input.state == ElementState::Pressed,
                            _ => ()
                        }
                    }
                    return
                }
                event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return
                },
                _ => return
            },
            _ => ()
        }
    })
}
    