mod models;         

use std::{ffi::{CString, c_void}, mem, os::raw::c_char, ptr::{self, null, null_mut}, sync::mpsc::Receiver};

use glfw::{Action, Context, Key};
use gl::types::*;
use models::shader::Shader;
use std::str;

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

    // glfw window creation
    let (mut window, events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // gl: load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let shader = Shader::new("assets/shaders/vertex.vert", "assets/shaders/fragment.frag");

    // vertices
    let vertices: [f32; 18] = [
        // positions      // colors
        -0.5, -0.5, 0.0,  1.0, 0.0, 0.0,
        0.5, -0.5, 0.0,  0.0, 1.0, 0.0,
        0.0,  0.5, 0.0,  0.0, 0.0, 1.0
    ];

    let indices: [GLuint; 3] = [
        0, 1, 2 // first triangle
        // 1, 2, 3 // second triangle
    ];

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
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 6 * std::mem::size_of::<GLfloat>() as GLsizei, ptr::null());
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 6 * std::mem::size_of::<GLfloat>() as GLsizei, (3 * std::mem::size_of::<GLfloat>()) as *const c_void);
    gl::EnableVertexAttribArray(1);

    // create element buffer
    let mut ebo = 0;
    gl::GenBuffers(1, &mut ebo);

    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER, 
        mem::size_of_val(&indices) as GLsizeiptr, 
        indices.as_ptr() as *const c_void, 
        gl::STATIC_DRAW
    );

    // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

    // render loop
    while !window.should_close() {
        // events
        process_events(&mut window, &events);

        // clear
        unsafe { gl::ClearColor(0.2, 0.3, 0.3, 1.0); }
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }

        // uniform
        // let time = glfw.get_time();
        // let green = time.sin() / 2.0 + 0.5;
        // let vertex_color_loc = unsafe { gl::GetUniformLocation(shader_program, CString::new("ourColor").unwrap().as_ptr()) };
        // unsafe { gl::Uniform4f(vertex_color_loc, 0.0, green as GLfloat, 0.0, 1.0) };

        // draw
        shader.set_float("alpha", 0.5);
        shader.use_program();
        unsafe { gl::DrawElements(gl::TRIANGLES, 3, gl::UNSIGNED_INT, ptr::null()); }

        // glfw: swap buffers and poll IO events
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe { gl::Viewport(0, 0, width, height) }
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            _ => ()
        }
    }
}