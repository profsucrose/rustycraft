mod models;      

use std::{ffi::c_void, mem, path::Path, ptr, sync::mpsc::Receiver, time::Instant};
use cgmath::{Matrix4, vec3};
use glfw::{Action, Context, CursorMode, Key, MouseButton, PixelImage, WindowEvent};
use gl::types::*;
use image::{RgbaImage, GenericImage};
use models::{core::player::Player, opengl::{framebuffer, tex_quad::TexQuad}};

use crate::models::{core::{block_type::BlockType, face::Face, world::World}, opengl::{camera::Camera, framebuffer::FrameBuffer, shader::Shader, text_renderer::TextRenderer, texture::Texture, vertex_array::VertexArray, vertex_buffer::VertexBuffer}};

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

    // create vertex array
    let vao = VertexArray::new(); 
    vao.bind();

    // create VBO
    let mut vbo = VertexBuffer::new();
    vbo.bind();

    // set vertex attribute pointers
    // position
    vbo.add_float_attribute(3, 10);
    for _ in 0..6 {
        // block indices attributes
        vbo.add_float_attribute(1, 10);
    }
    // faces to draw via bitwise
    vbo.add_float_attribute(1, 10);

    // create world object
    let mut world = World::new(20, "world");
    println!("Initialized new world");

    let texture_map = Texture::new(
        "assets/textures/textures.png", 
        gl::TEXTURE0, 
        false
    );

    let mut player = Player::new(SCR_WIDTH, SCR_HEIGHT);
    world.recalculate_mesh_from_perspective(0, 0);
    player.camera.position.y = world.highest_in_column(0, 0).unwrap() as f32 + 2.0;

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

    let mut selected_coords = None;

    let mut force_recalculation = false;

    let mut current_block_index = 0;

    let quad = TexQuad::new("assets/textures/water.png", gl::TEXTURE0, true, SCR_WIDTH, SCR_HEIGHT);

    // render loop
    while !window.should_close() {
        let deltatime = instant.elapsed().as_millis() as f32;
        instant = Instant::now();

        // events
        process_events(
            &mut window, 
            &events, 
            &mut mouse_captured,
            &selected_coords,
            &mut world,
            &mut player, 
            &mut last_x, 
            &mut last_y, 
            &mut first_mouse,
            &mut force_recalculation,
            &mut current_block_index
        );

        player.update_position(&world, deltatime);

        // update player altitude
        player.update_alt(&world);

        // bind off-screen framebuffer
        // framebuffer.bind();
        gl::Enable(gl::DEPTH_TEST);

        // clear buffers
        gl::ClearColor(29.0 / 255.0, 104.0 / 255.0, 224.0 / 255.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); 

        // draw tex
        text_renderer.render_text(format!("FPS: {}", (1000.0 / deltatime).round()).as_str(), 10.0, (SCR_HEIGHT as f32) - 30.0, 1.0, vec3(1.0, 1.0, 1.0));
        text_renderer.render_text(format!("x: {:.2}", player.camera.position.x).as_str(), 10.0, (SCR_HEIGHT as f32) - 50.0, 0.6, vec3(1.0, 1.0, 1.0));
        text_renderer.render_text(format!("y: {:.2}", player.camera.position.y).as_str(), 10.0, (SCR_HEIGHT as f32) - 70.0, 0.6, vec3(1.0, 1.0, 1.0));
        text_renderer.render_text(format!("z: {:.2}", player.camera.position.z).as_str(), 10.0, (SCR_HEIGHT as f32) - 90.0, 0.6, vec3(1.0, 1.0, 1.0));
        let block = match current_block_index {
            0 => BlockType::Dirt,
            1 => BlockType::Grass,
            2 => BlockType::Stone,
            3 => BlockType::Log,
            4 => BlockType::Leaves,
            5 => BlockType::Orange,
            6 => BlockType::DarkOrange,
            7 => BlockType::Black,
            _ => BlockType::Air
        };
        text_renderer.render_text(format!("Selected block: {:?}", block).as_str(), 10.0, (SCR_HEIGHT as f32) - 110.0, 0.6, vec3(1.0, 1.0, 1.0));

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
        shader.set_mat4("view", player.camera.get_view());
        shader.set_mat4("projection", player.camera.get_projection());
        shader.set_mat4("model", Matrix4::<f32>::from_scale(1.0));

        // bind texture
        texture_map.bind();
        shader.set_texture("texture_map", &texture_map);

        // draw
        vao.bind();
        vbo.bind();

        let meshes = world.get_world_mesh_from_perspective(player.camera.position.x as i32, player.camera.position.z as i32, force_recalculation);
        force_recalculation = false;
        // opaque block points
        for mesh in meshes.iter() {
            vbo.set_data(&mesh.0, gl::DYNAMIC_DRAW);
            gl::DrawArrays(gl::POINTS, 0, (mesh.0.len() / 10) as GLint);
        }
        
        // transparent block points
        for mesh in meshes.iter() {
            vbo.set_data(&mesh.1, gl::DYNAMIC_DRAW);
            gl::DrawArrays(gl::POINTS, 0, (mesh.1.len() / 10) as GLint);
        }
   
        selected_coords = world.raymarch_block(&player.camera.position, &player.camera.front);
        if let Some(((x, y, z), Some(face))) = selected_coords {
            let mut mesh = Vec::new();
            mesh.push(x as f32);
            mesh.push(y as f32);
            mesh.push(z as f32);
            for _ in 0..6 {
                mesh.push(7.0);
            };
            // let face = match i {
            //     0 => Face::Front,
            //     1 => Face::Right,
            //     2 => Face::Back,
            //     3 => Face::Bottom,
            //     4 => Face::Left,
            //     5 => Face::Top,
            //     _ => panic!("Attempted to convert invalid index to face when setting vertex texture UV indices")
            // }
            let face_to_draw = match face {
                Face::Front     => 0b10000000,
                Face::Right     => 0b01000000,
                Face::Back      => 0b00100000,
                Face::Bottom    => 0b00010000,
                Face::Left      => 0b00001000,
                Face::Top       => 0b00000100,
            };
            mesh.push(face_to_draw as f32);
            vbo.set_data(&mesh, gl::DYNAMIC_DRAW);

            shader.set_mat4("model", Matrix4::from_scale(1.01));
            gl::DrawArrays(gl::POINTS, 0, 1);
        }

        // couldn't get framebuffer to work for post-processing
        // so draw a blue textured transparent quad for underwater
        // effect now
        if player.underwater(&world) {
            player.camera.speed = 0.003;
            quad.draw(0.0, 0.0, SCR_WIDTH as f32, SCR_HEIGHT as f32, 0.7);
        } else {
            player.camera.speed = 0.008;
        }

        // draw to main framebuffer
        // framebuffer.draw();

        window.swap_buffers();
        glfw.poll_events();

        // hang thread for target FPS
        while (instant.elapsed().as_millis() as f32) < (1000.0 / target_fps) {}
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>, mouse_captured: &mut bool, selected_coords: &Option<((i32, i32, i32), Option<Face>)>, world: &mut World, player: &mut Player, last_x: &mut f32, last_y: &mut f32, first_mouse: &mut bool, force_recalculation: &mut bool, current_block_index: &mut u32) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            WindowEvent::FramebufferSize(width, height) => {
                unsafe { gl::Viewport(0, 0, width, height) }
            },
            WindowEvent::Scroll(_, y_offset) => {
                player.camera.scroll_callback(y_offset as f32);
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

                player.camera.mouse_callback(x_offset, y_offset);
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
                if let Some(((x, y, z), _)) = selected_coords {
                    world.set_block(*x, *y, *z, BlockType::Air);
                    *force_recalculation = true;
                    //world.recalculate_mesh_from_perspective((camera.position.x as i32) % 16, (camera.position.z as i32) % 16);
                }
            },
            WindowEvent::MouseButton(MouseButton::Button2, Action::Press, _) => {
                if let Some(((x, y, z), Some(face))) = selected_coords {
                    let x = *x;
                    let y = *y;
                    let z = *z;
                    
                    let place_position = match face {
                        Face::Top => (x, y + 1, z),
                        Face::Bottom => (x, y - 1, z),
                        Face::Right => (x + 1, y, z),
                        Face::Left => (x - 1, y, z),
                        Face::Front => (x, y, z - 1),
                        Face::Back => (x, y, z + 1),
                    };
                    let block = match current_block_index {
                        0 => BlockType::Dirt,
                        1 => BlockType::Grass,
                        2 => BlockType::Stone,
                        3 => BlockType::Log,
                        4 => BlockType::Leaves,
                        5 => BlockType::Orange,
                        6 => BlockType::DarkOrange,
                        7 => BlockType::Black,
                        _ => BlockType::Air
                    };

                    if place_position.0 == player.camera.position.x.round() as i32
                        && (place_position.1 == player.camera.position.y.round() as i32
                        || place_position.1 == player.camera.position.y.round() as i32 - 1)
                        && place_position.2 == player.camera.position.z.round() as i32 {
                            return;
                        }
                    world.set_block(place_position.0, place_position.1, place_position.2, block);
                    *force_recalculation = true;
                    //world.recalculate_mesh_from_perspective((camera.position.x as i32) % 16, (camera.position.z as i32) % 16);
                }
            }, 
            WindowEvent::Key(Key::Space, _, Action::Press, _) => player.jump(),
            WindowEvent::Key(Key::Up, _, Action::Press, _) => *current_block_index += 1,
            WindowEvent::Key(Key::Down, _, Action::Press, _) => if *current_block_index > 0 { *current_block_index -= 1 },
            WindowEvent::Key(Key::LeftShift, _, Action::Press, _) => player.camera.speed = 0.05,
            WindowEvent::Key(Key::LeftShift, _, Action::Release, _) => player.camera.speed = 0.008,
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
                player.camera.process_keyboard(key, action);
            },
            _ => ()
        }
    }
}