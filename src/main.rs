mod models;      

use std::{ffi::c_void, mem, path::Path, ptr, sync::mpsc::Receiver, time::Instant};
use cgmath::{Matrix4, vec3};
use glfw::{Action, Context, CursorMode, Key, MouseButton, PixelImage, WindowEvent};
use gl::types::*;
use models::{block_type::BlockType, camera::Camera, shader::Shader, text_renderer::TextRenderer, texture::Texture, vertex_array::VertexArray, vertex_buffer::VertexBuffer, world::World};
use image::{RgbaImage, GenericImage};

// settings
const SCR_WIDTH: u32 = 1000;
const SCR_HEIGHT: u32 = 600;

fn main() {
    // wrap program in helper
    // for unsafe block w/o indentation
    unsafe { start(); }
}

unsafe fn start() {
    // glfw: initialize
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true)); 

    // glfw window creation
    let (mut window, events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "RustyCraft", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_scroll_polling(true);
    window.set_mouse_button_polling(true);
    window.set_title("RustyCraft");

    // set window icon; note that on MacOS this does nothing
    // as the icon must be set via .app bundling
    let icon = image::open(&Path::new("assets/textures/icon.png"))
        .expect("Failed to load texture");
    let pixels = icon.raw_pixels().iter().map(|pixel| *pixel as u32).collect();
    let glfw_image = PixelImage {
        width: icon.width(),
        height: icon.height(),
        pixels
    };
    window.set_icon_from_pixels(vec![glfw_image]);

    // capture mouse
    let mut mouse_captured = true;
    window.set_cursor_mode(CursorMode::Disabled);

    // gl: load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // depth buffer
    gl::Enable(gl::DEPTH_TEST);

    // blending
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

    let shader = Shader::new_with_geom("assets/shaders/vertex.vert", "assets/shaders/fragment.frag", "assets/shaders/geometry.geom");

    // crosshairs
    // let crosshairs_texture = Texture::new("assets/textures/crosshairs.png", gl::TEXTURE0, true);
    // let crosshairs_vao = VertexArray::new();
    // crosshairs_vao.bind();

    // let crosshairs_shader = Shader::new("assets/shaders/crosshairs.vert", "assets/shaders/crosshairs.frag");
    // let mut crosshairs_vbo = VertexBuffer::new();
    // crosshairs_vbo.bind_to_array_buffer();
    
    // position
    // crosshairs_vbo.add_float_attribute(2, 4);

    // texcoords
    // crosshairs_vbo.add_float_attribute(2, 4);

    // let crosshairs_vertices: Vec<f32> = vec![
    //      0.01,  0.01, 1.0, 1.0, // top right
    //      0.01, -0.01, 1.0, 0.0, // bottom right
    //     -0.01,  0.01, 0.0, 1.0, // top left
    //     -0.01, -0.01, 0.0, 0.0, // bottom left
    // ];

    // let middle_x = (SCR_WIDTH / 2) as f32;
    // let middle_y = (SCR_HEIGHT / 2) as f32;

    // let crosshairs_vertices: Vec<f32> = vec![
    //       middle_x + 15.0,    middle_y + 15.0, 1.0, 1.0, // top right
    //       middle_x + 15.0,    middle_y, 1.0, 0.0, // bottom right
    //       middle_x,    15.0, 0.0, middle_y + 15.0, // top left
    //       middle_x,    0.0, 0.0, middle_y // bottom left
    // ];
    // crosshairs_vbo.set_data(&crosshairs_vertices, gl::STATIC_DRAW);
    
    // create vertex array
    let vao = VertexArray::new(); 
    vao.bind();

    // create VBO
    let mut vbo = VertexBuffer::new();
    vbo.bind();

    // set vertex attribute pointers
    // position
    vbo.add_float_attribute(3, 9);
    for _ in 0..6 {
        // block indices attributes
        vbo.add_float_attribute(1, 9);
    }

    // create world object
    let mut world = World::new(10);
    println!("Initialized new world");

    let texture_map = Texture::new(
        "assets/textures/textures.png", 
        gl::TEXTURE0, 
        false
    );

    let mut camera = Camera::new(SCR_WIDTH, SCR_HEIGHT, 0.008);

    let mut instant = Instant::now();

    // last mouse x and y coords
    let mut last_x = 400.0;
    let mut last_y = 300.0;

    // store if the mouse has entered the window to prevent
    // initially yanking the camera
    let mut first_mouse = true;

    // init text renderer
    let text_renderer = TextRenderer::new(SCR_WIDTH, SCR_HEIGHT, "assets/font/OldSchoolAdventures.ttf", "assets/shaders/text_vertex.vert", "assets/shaders/text_fragment.frag");

    // target fps
    let target_fps = 60.0;

    // render loop
    while !window.should_close() {
        let deltatime = instant.elapsed().as_millis() as f32;
        instant = Instant::now();

        // events
        process_events(
            &mut window, 
            &events, 
            &mut mouse_captured,
            &mut world,
            &mut camera, 
            &mut last_x, 
            &mut last_y, 
            &mut first_mouse
        );
        camera.update_position(deltatime);

        // clear buffers
        gl::ClearColor(29.0 / 255.0, 104.0 / 255.0, 224.0 / 255.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); 

        // draw tex
        text_renderer.render_text(format!("FPS: {}", (1000.0 / deltatime).round()).as_str(), 10.0, (SCR_HEIGHT as f32) - 30.0, 1.0, vec3(1.0, 0.0, 0.0));
        text_renderer.render_text(format!("x: {:.2}", camera.position.x).as_str(), 10.0, (SCR_HEIGHT as f32) - 50.0, 0.6, vec3(1.0, 0.0, 0.0));
        text_renderer.render_text(format!("y: {:.2}", camera.position.y).as_str(), 10.0, (SCR_HEIGHT as f32) - 70.0, 0.6, vec3(1.0, 0.0, 0.0));
        text_renderer.render_text(format!("z: {:.2}", camera.position.z).as_str(), 10.0, (SCR_HEIGHT as f32) - 90.0, 0.6, vec3(1.0, 0.0, 0.0));

        // draw crosshairs
        // crosshairs_shader.use_program();
        // crosshairs_shader.set_mat4(
        //     "orthographic", 
        // cgmath::ortho(0.0, SCR_WIDTH as f32, 0.0, SCR_HEIGHT as f32, -1.0, 100.0)
        // );
        // crosshairs_texture.bind();
        // crosshairs_shader.set_texture("tex", &crosshairs_texture);
        // crosshairs_vao.bind();
        // crosshairs_vbo.bind_to_array_buffer();
        // gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);

        // shader uniforms
        shader.use_program();
        // transforms
        shader.set_mat4("view", camera.get_view());
        shader.set_mat4("projection", camera.get_projection());
        shader.set_mat4("model", Matrix4::<f32>::from_scale(1.0));

        // bind texture
        texture_map.bind();
        shader.set_texture("texture_map", &texture_map);

        // draw
        vao.bind();
        vbo.bind();

        let meshes = world.get_world_mesh_from_perspective(camera.position.x as i32, camera.position.z as i32);
        for mesh in meshes.iter() {
            vbo.set_data(&mesh, gl::DYNAMIC_DRAW);
            gl::DrawArrays(gl::POINTS, 0, (mesh.len() / 9) as GLint);
        }

        let result = world.raymarch_block(&camera.position, &camera.front);
        if let Some((x, y, z)) = result {
            let x = x as f32;
            let y = y as f32; 
            let z = z as f32;
            let mut mesh = vec![x, y, z];
            for _ in 0..6 {
                mesh.push(7.0);
            }
            println!("Rendering cube at {} {} {}", x, y, z);
            shader.set_mat4("model", Matrix4::from_scale(1.001));

            vbo.set_data(&mesh, gl::DYNAMIC_DRAW);
            gl::DrawArrays(gl::POINTS, 0, 1);
        }
        window.swap_buffers();
        glfw.poll_events();

        // hang thread for target FPS
        while (instant.elapsed().as_millis() as f32) < (1000.0 / target_fps) {}
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>, mouse_captured: &mut bool, world: &mut World, camera: &mut Camera, last_x: &mut f32, last_y: &mut f32, first_mouse: &mut bool) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            WindowEvent::FramebufferSize(width, height) => {
                unsafe { gl::Viewport(0, 0, width, height) }
            },
            WindowEvent::Scroll(_, y_offset) => {
                camera.scroll_callback(y_offset as f32);
            },
            WindowEvent::CursorPos(xpos, ypos) => {
                let (x_pos, y_pos) = (xpos as f32, ypos as f32);
                let x_offset = x_pos - *last_x;
                let y_offset = *last_y - y_pos;
                *last_x = x_pos;
                *last_y = y_pos;
                if *first_mouse {
                    *first_mouse = false;
                    return;
                }

                if !*mouse_captured {
                    return;
                }

                camera.mouse_callback(x_offset, y_offset);
            },
            WindowEvent::Key(Key::F2, _, Action::Press, _) => {
                let width = SCR_WIDTH;
                let height = SCR_HEIGHT;
                let mut data = vec![0u8; (width * height * 4) as usize].into_boxed_slice();
                unsafe { gl::ReadPixels(0, height as i32, width as i32, height as i32, gl::RGBA, gl::UNSIGNED_BYTE, data.as_mut_ptr() as *mut c_void); }

                let image = RgbaImage::from_raw(width, height, data.to_vec())
                    .expect("Unable to convert pixel array to RgbImage");
                image.save("screenshot.png").expect("Unable to write image to file");
                println!("Saved screenshot");
            },
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                // println!("Mouse button event");
                let result = world.raymarch_block(&camera.position, &camera.front);
                if let Some((x, y, z)) = result {
                    world.set_block(x, y, z, BlockType::Log);
                    world.recalculate_mesh_from_perspective((camera.position.x as i32) % 16, (camera.position.z as i32) % 16)
                }
                println!("Raymarched block from pos: {:?}", result);
            },
            WindowEvent::Key(Key::LeftShift, _, Action::Press, _) => camera.speed = 0.05,
            WindowEvent::Key(Key::LeftShift, _, Action::Release, _) => camera.speed = 0.008,
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            WindowEvent::Key(Key::LeftSuper, _, Action::Press, _) => {
                println!("Toggling window focus");
                *mouse_captured = !*mouse_captured;
                window.set_cursor_mode(match *mouse_captured {
                    true => CursorMode::Disabled,
                    false => CursorMode::Normal
                });
            },
            WindowEvent::Key(key, _, action, _) => {
                camera.process_keyboard(key, action);
            },
            _ => ()
        }
    }
}