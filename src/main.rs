mod models;         

use std::{ffi::c_void, mem, ptr, sync::mpsc::Receiver, time::Instant};

use cgmath::{Deg, Matrix4, Vector3, vec3};
use cgmath::prelude::*;
use glfw::{Action, Context, Key};
use gl::types::*;
use image::{ImageFormat};
use models::{camera::Camera, shader::Shader, texture};
use texture::load_texture;

// settings
const SCR_WIDTH: u32 = 800;
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
    glfw.window_hint(glfw::WindowHint::Samples(Some(4))); 

    // glfw window creation
    let (mut window, events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_scroll_polling(true);

    // capture mouse
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    // glfw: lock cursor
    // glfw::ffi::glfwSetInputMode(
    //     window.window_ptr(), 
    //     glfw::ffi::CURSOR, 
    //     glfw::ffi::CURSOR_DISABLED
    // );

    // gl: load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // depth buffer
    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::MULTISAMPLE);

    let shader = Shader::new("assets/shaders/vertex.vert", "assets/shaders/fragment.frag");

    // vertices
    let vertices: [f32; 180] = [
        -0.5, -0.5, -0.5,  0.0, 0.0,
         0.5, -0.5, -0.5,  1.0, 0.0,
         0.5,  0.5, -0.5,  1.0, 1.0,
         0.5,  0.5, -0.5,  1.0, 1.0,
        -0.5,  0.5, -0.5,  0.0, 1.0,
        -0.5, -0.5, -0.5,  0.0, 0.0,
    
        -0.5, -0.5,  0.5,  0.0, 0.0,
         0.5, -0.5,  0.5,  1.0, 0.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
        -0.5,  0.5,  0.5,  0.0, 1.0,
        -0.5, -0.5,  0.5,  0.0, 0.0,
    
        -0.5,  0.5,  0.5,  1.0, 0.0,
        -0.5,  0.5, -0.5,  1.0, 1.0,
        -0.5, -0.5, -0.5,  0.0, 1.0,
        -0.5, -0.5, -0.5,  0.0, 1.0,
        -0.5, -0.5,  0.5,  0.0, 0.0,
        -0.5,  0.5,  0.5,  1.0, 0.0,
    
         0.5,  0.5,  0.5,  1.0, 0.0,
         0.5,  0.5, -0.5,  1.0, 1.0,
         0.5, -0.5, -0.5,  0.0, 1.0,
         0.5, -0.5, -0.5,  0.0, 1.0,
         0.5, -0.5,  0.5,  0.0, 0.0,
         0.5,  0.5,  0.5,  1.0, 0.0,
    
        -0.5, -0.5, -0.5,  0.0, 1.0,
         0.5, -0.5, -0.5,  1.0, 1.0,
         0.5, -0.5,  0.5,  1.0, 0.0,
         0.5, -0.5,  0.5,  1.0, 0.0,
        -0.5, -0.5,  0.5,  0.0, 0.0,
        -0.5, -0.5, -0.5,  0.0, 1.0,
    
        -0.5,  0.5, -0.5,  0.0, 1.0,
         0.5,  0.5, -0.5,  1.0, 1.0,
         0.5,  0.5,  0.5,  1.0, 0.0,
         0.5,  0.5,  0.5,  1.0, 0.0,
        -0.5,  0.5,  0.5,  0.0, 0.0,
        -0.5,  0.5, -0.5,  0.0, 1.0
    ];

    let cube_positions: [Vector3<f32>; 10] = [
        vec3( 0.0,  0.0,  0.0), 
        vec3( 2.0,  5.0, -15.0), 
        vec3(-1.5, -2.2, -2.5),  
        vec3(-3.8, -2.0, -12.),  
        vec3( 2.4, -0.4, -3.5),  
        vec3(-1.7,  3.0, -7.5),  
        vec3( 1.3, -2.0, -2.5),  
        vec3( 1.5,  2.0, -2.5), 
        vec3( 1.5,  0.2, -1.5), 
        vec3(-1.3,  1.0, -1.5)  
    ];

    // let indices: [GLuint; 6] = [
    //     0, 1, 3, // first triangle
    //     1, 2, 3 // second triangle
    // ];

    // create vertex array
    let mut vao = 0;
    gl::GenVertexArrays(1, &mut vao);

    // bind VAO
    gl::BindVertexArray(vao);

    // create vertex buffer
    let mut vbo = 0;
    gl::GenBuffers(1, &mut vbo);

    // copy vertices data to buffer
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER, 
        mem::size_of_val(&vertices) as isize, 
        &vertices[0] as *const f32 as *const c_void, 
        gl::STATIC_DRAW
    );

    // set vertex attribute pointers
    // position
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 5 * std::mem::size_of::<GLfloat>() as GLsizei, ptr::null());
    gl::EnableVertexAttribArray(0);
    // texcoords
    gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 5 * std::mem::size_of::<GLfloat>() as GLsizei, (3 * std::mem::size_of::<GLfloat>()) as *const c_void);
    gl::EnableVertexAttribArray(1);

    // create element buffer
    let mut ebo = 0;
    gl::GenBuffers(1, &mut ebo);

    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

    gl::BufferData(
        gl::ARRAY_BUFFER, 
        (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
        &vertices[0] as *const f32 as *const c_void,
        gl::STATIC_DRAW
    );

    let texture1 = load_texture(
        "assets/textures/container.jpg", 
        gl::TEXTURE0, 
        false
    );
    let texture2 = load_texture(
        "assets/textures/awesomeface.png", 
        gl::TEXTURE1, 
        true
    );
    let texture3 = load_texture(
        "assets/textures/grass.png", 
        gl::TEXTURE2, 
        true
    );

    let mut camera = Camera::new();
    let mut instant = Instant::now();

    let mut last_x = 400.0;
    let mut last_y = 300.0;

    let mut first_mouse = true;
 
    // render loop
    while !window.should_close() {
        let deltatime = instant.elapsed().as_millis() as f32;
        instant = Instant::now();

        // events
        process_events(
            &mut window, 
            &events, 
            &mut camera, 
            deltatime, 
            &mut last_x, 
            &mut last_y, 
            &mut first_mouse
        );
        camera.update_position(deltatime);

        // clear
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); 

        // uniform
        // let time = glfw.get_time();
        // let green = time.sin() / 2.0 + 0.5;
        // let vertex_color_loc = unsafe { gl::GetUniformLocation(shader_program, CString::new("ourColor").unwrap().as_ptr()) };
        // unsafe { gl::Uniform4f(vertex_color_loc, 0.0, green as GLfloat, 0.0, 1.0) };

        // transforms
        shader.set_mat4("view", camera.get_view());
        shader.set_mat4("projection", camera.get_projection());
        for i in 0..10 {
            let angle = 20.0 * (i as f32);
            let model: Matrix4<f32> = Matrix4::<f32>::from_translation(cube_positions[i])
                * Matrix4::<f32>::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Deg(angle));
            shader.set_mat4("model", model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        // let mut transform: Matrix4<f32> = Matrix4::identity();
        // transform = transform * Matrix4::<f32>::from_translation(vec3(0.5, -0.5, 0.0));
        // transform = transform * Matrix4::<f32>::from_angle_z(Rad(glfw.get_time() as f32));

        // draw
        shader.use_program();
        shader.set_uint("texture1", texture1);
        shader.set_uint("texture2", texture2);
        shader.set_uint("texture3", texture3);

        //gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null()); 

        // glfw: swap buffers and poll IO events
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>, camera: &mut Camera, deltatime: f32, last_x: &mut f32, last_y: &mut f32, first_mouse: &mut bool) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe { gl::Viewport(0, 0, width, height) }
            },
            glfw::WindowEvent::Scroll(_, y_offset) => {
                camera.scroll_callback(y_offset as f32);
            },
            glfw::WindowEvent::CursorPos(xpos, ypos) => {
                let (x_pos, y_pos) = (xpos as f32, ypos as f32);
                let x_offset = x_pos - *last_x;
                let y_offset = *last_y - y_pos;
                *last_x = x_pos;
                *last_y = y_pos;
                //println!("Moved mouse: {} {}", x_offset, y_offset);
                if *first_mouse {
                    *first_mouse = false;
                    return;
                }
                camera.mouse_callback(x_offset, y_offset);
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            glfw::WindowEvent::Key(key, _, action, _) => camera.process_keyboard(key, action),
            _ => ()
        }
    }
}