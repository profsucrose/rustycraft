use std::{ffi::{CString, c_void}, mem, os::raw::c_char, ptr::{self, null, null_mut}, sync::mpsc::Receiver};

use glfw::{Action, Context, Key};
use gl::types::*;
use std::str;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

fn main() {
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

    let (vao, shader_program) = unsafe {
        let vertex_shader_src = "
            #version 330 core
            layout (location = 0) in vec3 aPos;

            void main() {
                gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
            }
        ";

        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vertex_shader, 1, &CString::new(vertex_shader_src.as_bytes()).unwrap().as_ptr(), null());
        gl::CompileShader(vertex_shader);

        // check if vertex shader compilation failed and if so check logs
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // subtract 1 to skip trailing null char
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(vertex_shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("VERTEX SHADER COMPILATION FAILED\n{:?}", str::from_utf8(&info_log).unwrap());
        }

        let fragment_shader_src = "
            #version 330 core
            out vec4 FragColor;

            uniform vec4 ourColor;

            void main() {
                FragColor = ourColor;
            }
        ";

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fragment_shader, 1, &CString::new(fragment_shader_src).unwrap().as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        // check if fragment shader compilation failed and if so check logs
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // subtract 1 to skip trailing null char
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(fragment_shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
            println!("FRAGMENT SHADER COMPILATION FAILED: {:?}", std::str::from_utf8(&info_log).unwrap());
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        // check if shader program linking failed and if so print logs
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1); // subtract 1 to skip trailing null char
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(shader_program, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut i8);
            println!("FRAGMENT SHADER COMPILATION FAILED: {:?}", std::str::from_utf8(&info_log).unwrap());
        }

        // use program
        gl::UseProgram(shader_program);

        // delete shaders
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        // vertices
        let vertices: [f32; 9] = [
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
            0.0, 0.5, 0.0
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
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * std::mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(0);

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
        (vao, shader_program)   
    };

    // render loop
    while !window.should_close() {
        // events
        process_events(&mut window, &events);

        // clear
        unsafe { gl::ClearColor(0.2, 0.3, 0.3, 1.0); }
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }

        // uniform
        let time = glfw.get_time();
        let green = time.sin() / 2.0 + 0.5;
        let vertex_color_loc = unsafe { gl::GetUniformLocation(shader_program, CString::new("ourColor").unwrap().as_ptr()) };
        unsafe { gl::Uniform4f(vertex_color_loc, 0.0, green as GLfloat, 0.0, 1.0) };

        // draw
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