mod models;  
mod utils;       

use std::{collections::HashMap, ffi::c_void, mem, ptr, sync::mpsc::Receiver, time::Instant};

use cgmath::{Deg, Matrix4, Vector3, ortho, vec3};
use cgmath::prelude::*;
use freetype::Library;
use glfw::{Action, Context, Key};
use gl::types::*;
use image::{ImageFormat};
use models::{camera::Camera, character::Character, shader::Shader, texture::Texture, world::World};
use utils::{character_map::gen_character_map, cube::cube_vertices, texture::load_texture};

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
    //glfw.window_hint(glfw::WindowHint::Samples(Some(4))); 

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

    // gl: load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // depth buffer
    gl::Enable(gl::DEPTH_TEST);
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

    let shader = Shader::new("assets/shaders/vertex.vert", "assets/shaders/fragment.frag");
    let text_shader = Shader::new("assets/shaders/text_vertex.vert", "assets/shaders/text_fragment.frag");

    // vertices
    let vertices = cube_vertices();

    let offsets: [Vector3<f32>; 3] = [
        vec3(0.0,  0.0,  0.0),
        vec3(1.0,  0.0,  0.0),
        vec3(2.0,  0.0,  0.0)
    ];

    let mut text_vao = 0;
    gl::GenVertexArrays(1, &mut text_vao);
    gl::BindVertexArray(text_vao);
    let mut text_vbo = 0;
    gl::GenBuffers(1, &mut text_vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, text_vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER, 
        6 * 4 * std::mem::size_of::<GLfloat>() as isize,
        ptr::null(),
        gl::DYNAMIC_DRAW
    );
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(
        0, 
        4, 
        gl::FLOAT, 
        gl::FALSE, 
        4 * std::mem::size_of::<GLfloat>() as i32, 
        ptr::null()
    );
    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    gl::BindVertexArray(0);
    
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
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 6 * std::mem::size_of::<GLfloat>() as GLsizei, ptr::null());
    gl::EnableVertexAttribArray(0);
    // texcoords
    gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 6 * std::mem::size_of::<GLfloat>() as GLsizei, (3 * std::mem::size_of::<GLfloat>()) as *const c_void);
    gl::EnableVertexAttribArray(1);
    // block index
    gl::VertexAttribPointer(2, 1, gl::FLOAT, gl::FALSE, 6 * std::mem::size_of::<GLfloat>() as GLsizei, (5 * std::mem::size_of::<GLfloat>()) as *const c_void);
    gl::EnableVertexAttribArray(2);

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

    let grass_texture = Texture::new(
        "assets/textures/grass.png", 
        gl::TEXTURE1, 
        true
    );
    let dirt_texture = Texture::new(
        "assets/textures/dirt.png", 
        gl::TEXTURE0, 
        false
    );

    let mut camera = Camera::new();
    camera.position.y = 20.0;

    let mut instant = Instant::now();

    let mut last_x = 400.0;
    let mut last_y = 300.0;

    let mut first_mouse = true;
 
    let world = World::new();

    // init library
    let lib = Library::init().unwrap();
    
    // load a font facer
    let face = lib.new_face("assets/font/OldSchoolAdventures.ttf", 0).unwrap();

    // font size
    face.set_pixel_sizes(0, 20).unwrap();

    // disable byte-alignment restriction
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

    let char_map = gen_character_map(&face);

    let target_fps = 60.0;

    let mut counter = 0;
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

        // clear buffers
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); 

        // draw text
        render_text(
            &text_shader, 
            &char_map, 
            text_vao, 
            text_vbo, 
            format!("FPS: {}", (1000.0 / deltatime).round()).as_str(), 
            10.0, 
            10.0, 
            1.0, 
            vec3(1.0, 0.0, 0.0)
        );
        render_text(
            &text_shader, 
            &char_map, 
            text_vao, 
            text_vbo, 
            "Hello World!",
            10.0, 
            50.0, 
            1.0, 
            vec3(0.0, 1.0, 0.0)
        );
        render_text(
            &text_shader, 
            &char_map, 
            text_vao, 
            text_vbo, 
            format!("{}", counter / 60).as_str(),
            SCR_WIDTH as f32 / 2.0 - 50.0, 
            SCR_HEIGHT as f32 - 50.0, 
            0.37, 
            vec3(0.0, 1.0, 0.0)
        );
        counter += 1;

        // shader uniforms
        shader.use_program();
        // shader.set_texture("grass_texture", &grass_texture);
        // shader.set_texture("dirt_texture", &dirt_texture);

        // transforms
        shader.set_mat4("view", camera.get_view());
        shader.set_mat4("projection", camera.get_projection());

        // draw
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        dirt_texture.bind();
        grass_texture.bind();
        shader.set_texture("dirtTexture", &dirt_texture);
        shader.set_texture("grassTexture", &grass_texture);

        let model_vectors = world.to_model_vectors();
        for (i, offset) in model_vectors.iter().enumerate() {
            let model: Matrix4<f32> = Matrix4::<f32>::from_scale(0.6)
                * Matrix4::<f32>::from_translation(*offset);
            shader.set_mat4("model", model);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        window.swap_buffers();
        glfw.poll_events();

        // hang thread for target FPS
        while (instant.elapsed().as_millis() as f32) < (1000.0 / target_fps) {}
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

unsafe fn render_text(shader: &Shader, char_map: &HashMap<usize, Character>, vao: u32, vbo: u32, text: &str, mut x: f32, y: f32, scale: f32, color: Vector3<f32>) {
    shader.use_program();
    shader.set_mat4("projection", ortho(0.0, 800.0, 0.0, 600.0, -1.0, 100.0));
    shader.set_vec3("textColor", color);
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindVertexArray(vao);

    for c in text.bytes() {
        let ch = &char_map[&(c as usize)];

        let x_pos = x + (ch.bearing.x as f32) * scale;
        let y_pos = y;// - ((ch.size.x - ch.bearing.y) as f32) * scale;

        let w = (ch.size.x as f32) * scale;
        let h = (ch.size.y as f32) * scale;

        // generate vertices for charatcer
        let vertices: [[f32; 4]; 6] = [
            [ x_pos,     y_pos + h,  0.0, 0.0 ],
            [ x_pos,     y_pos,      0.0, 1.0 ],
            [ x_pos + w, y_pos,      1.0, 1.0 ],

            [ x_pos,     y_pos + h,  0.0, 0.0 ],
            [ x_pos + w, y_pos,      1.0, 1.0 ],
            [ x_pos + w, y_pos + h,  1.0, 0.0 ]
        ];

        // render glyph texture over quad
        gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);

        // update VBO memory
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferSubData(gl::ARRAY_BUFFER, 0, std::mem::size_of_val(&vertices) as isize, vertices.as_ptr() as *const c_void);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        // render quad
        gl::DrawArrays(gl::TRIANGLES, 0, 6);

        x += ((ch.advance >> 6) as f32) * scale;
    }
    gl::BindVertexArray(0);
    gl::BindTexture(gl::TEXTURE_2D, 0);
}