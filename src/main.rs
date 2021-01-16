#![allow(unused_imports)]
mod models;      

use std::{ffi::c_void, fs, mem, path::Path, ptr, sync::mpsc::Receiver, time::Instant};
use cgmath::{Deg, Matrix4, Vector3, vec3};
use glfw::{Action, Context, CursorMode, Key, MouseButton, PixelImage, WindowEvent};
use gl::types::*;
use image::{RgbaImage, GenericImage};
use models::{core::player::Player, opengl::{tex_quad::TexQuad}};

use crate::models::{core::{block_type::BlockType, face::Face, window_mode::WindowMode, world::World}, opengl::{button::Button, camera::Camera, framebuffer::FrameBuffer, input::Input, shader::Shader, text_renderer::TextRenderer, texture::Texture, vertex_array::VertexArray, vertex_buffer::VertexBuffer}};

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
    let (mut window, events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "Rus", glfw::WindowMode::Windowed)
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
    // window.set_cursor_mode(CursorMode::Disabled);

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

    let texture_map = Texture::new(
        "assets/textures/textures.png", 
        gl::TEXTURE0, 
        false
    );

    let mut player = Player::new(SCR_WIDTH, SCR_HEIGHT);

    let mut instant = Instant::now();

    // last mouse x and y coords
    let mut last_x: f32 = 400.0;
    let mut last_y: f32 = 300.0;

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

    let mut menu_world = World::new_with_seed(10, "menu_world", 0);

    let mut menu_camera = Camera::new(SCR_WIDTH, SCR_HEIGHT, 0.0);
    menu_world.recalculate_mesh_from_perspective(0, 0);
    menu_camera.position.y = menu_world.highest_in_column(0, 0).unwrap() as f32 + 10.0;
    menu_camera.fov = 60.0;
    menu_camera.mouse_callback(0.0, -140.0);

    let mut yellow_text_size: f32 = 0.0;

    let button_width = 420.0;
    let button_height = 48.0;
    let button_x = SCR_WIDTH as f32 / 2.0;
    let select_worlds_button = Button::new("Open World", button_x, 280.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);
    let open_world_button = Button::new("Open", button_x, 210.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);
    let connect_button = Button::new("Connect", button_x, 210.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);
    let back_button = Button::new("Back", button_x, 140.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);
    let connect_to_server_button = Button::new("Connect to Server", button_x, 210.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);
    let mut open_world_input = Input::new(button_x, 280.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);

    let last_world = fs::read_to_string(".last_world");
    if last_world.is_ok() {
        open_world_input.text = last_world.unwrap();
    }

    let mut connect_to_server_input = Input::new(button_x, 280.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);

    // window
    let mut window_mode = WindowMode::Title;

    // placeholder world object
    let mut world = None;

    // render loop
    while !window.should_close() {
        let deltatime = instant.elapsed().as_millis() as f32;
        instant = Instant::now();

        // clear buffers
        gl::ClearColor(29.0 / 255.0, 104.0 / 255.0, 224.0 / 255.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); 
        match window_mode {
            WindowMode::Title | WindowMode::OpenWorld | WindowMode::ConnectToServer => {
                for (_, event) in glfw::flush_messages(&events) {
                    match event {
                        WindowEvent::FramebufferSize(width, height) => {
                            gl::Viewport(0, 0, width, height) 
                        },
                        WindowEvent::CursorPos(x, y) => {
                            last_x = x as f32;
                            last_y = y as f32;
                        },
                        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                            println!("Clicked");
                            let last_y = SCR_HEIGHT as f32 - last_y;
                            match window_mode {
                                WindowMode::Title => {
                                    if select_worlds_button.is_hovered(last_x, last_y) {
                                        println!("Select worlds clicked!");
                                        window_mode = WindowMode::OpenWorld;
                                    }
        
                                    if connect_to_server_button.is_hovered(last_x, last_y) {
                                        println!("Connect to server clicked!");
                                        window_mode = WindowMode::ConnectToServer;
                                    }
                                },
                                WindowMode::OpenWorld => {
                                    open_world_input.update_focus(last_x, last_y);
                                    if back_button.is_hovered(last_x, last_y) {
                                        window_mode = WindowMode::Title;
                                    }

                                    if open_world_button.is_hovered(last_x, last_y) {
                                        let mut world_object = World::new(13, open_world_input.text.clone().as_str());
                                        world_object.recalculate_mesh_from_perspective(0, 0);
                                        player.camera.position.y = world_object.highest_in_column(0, 0).unwrap() as f32 + 2.0;
                                        world = Some(world_object);
                                        window_mode = WindowMode::InWorld;
                                        window.set_cursor_mode(CursorMode::Disabled);
                                        fs::write(".last_world", open_world_input.text.clone())
                                            .expect("Failed to write world input text to file");
                                    }
                                },
                                WindowMode::ConnectToServer => {
                                    connect_to_server_input.update_focus(last_x, last_y);
                                    if back_button.is_hovered(last_x, last_y) {
                                        window_mode = WindowMode::Title;
                                    }
                                },
                                _ => ()
                            }
                        },
                        WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
                        WindowEvent::Key(key, _, Action::Press, _) => {
                            match window_mode {
                                WindowMode::OpenWorld => {
                                    open_world_input.type_key(key);
                                },
                                WindowMode::ConnectToServer => {
                                    connect_to_server_input.type_key(key);
                                },
                                _ => ()
                            }
                        },
                        _ => ()
                    }
                }

                // if window mode was changed to InWorld:
                if window_mode == WindowMode::InWorld {
                    continue;
                }
    
                // text_renderer.render_text("Create World", x + 20.0, 200.0, 1.0, Vector3::new(1.0, 0.0, 0.0));
                // button.draw(x, 180.0, x + 200.0, 230.0, 1.0);
    
                yellow_text_size += 0.075;
    
                // shader uniforms
                shader.use_program();
                // transforms
                shader.set_mat4("view", menu_camera.get_view());
                shader.set_mat4("projection", menu_camera.get_projection());
                shader.set_mat4("model", Matrix4::<f32>::from_scale(1.0));
    
                menu_camera.mouse_callback(0.15, 0.0);
    
                // bind texture
                texture_map.bind();
                shader.set_texture("texture_map", &texture_map);
    
                // draw
                vao.bind();
                vbo.bind();
    
                let meshes = menu_world.get_world_mesh_from_perspective(0, 0, false);
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
    
                // text
                let x = (SCR_WIDTH / 2) as f32;
                let y = SCR_HEIGHT as f32 - 220.0;
                let subtitle_size = 0.9 + (yellow_text_size.sin() + 1.0) / 30.0;
                text_renderer.render_text_with_mat("Shitty Minecraft -- in Rust!", 900.0 - subtitle_size * 100.0, y - 130.0, subtitle_size, Vector3::new(1.0, 1.0, 0.0), Matrix4::<f32>::from_angle_z(Deg(10.0)), true);
                text_renderer.render_text_centered("RustyCraft", x, y, 3.5, Vector3::new(1.0, 0.0, 0.0));
                text_renderer.render_text_centered("RustyCraft", x + 3.0, y - 3.0, 3.5, Vector3::new(173.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0));
                text_renderer.render_text("v0.1", SCR_WIDTH as f32 - 55.0, 10.0, 0.9, Vector3::new(1.0, 1.0, 1.0));
   
                let last_y = SCR_HEIGHT as f32 - last_y;
                match window_mode {
                    WindowMode::Title => {
                        select_worlds_button.draw(&text_renderer, last_x, last_y);
                        connect_to_server_button.draw(&text_renderer, last_x, last_y);
                    },
                    WindowMode::OpenWorld => {
                        open_world_button.draw(&text_renderer, last_x, last_y);
                        open_world_input.draw(&text_renderer);
                        back_button.draw(&text_renderer, last_x, last_y);
                    },
                    WindowMode::ConnectToServer => {
                        connect_button.draw(&text_renderer, last_x, last_y);
                        connect_to_server_input.draw(&text_renderer);
                        back_button.draw(&text_renderer, last_x, last_y);
                    },
                    _ => panic!("Attempted to display window mode without a title menu")
                };
                
                open_world_input.draw(&text_renderer);
            },
            // WindowMode::WorldMenu => {
            //     // text_renderer.render_text("World Menu", 0.0, 0.0, 1.0, Vector3::new(1.0, 0.0, 0.0));
            //     select_world_background.draw(0.0, 0.0, SCR_WIDTH as f32, SCR_HEIGHT as f32, 1.0);
            // },
            WindowMode::InWorld => {
                let mut world = world.as_mut().unwrap();

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
                    &mut current_block_index,
                    &mut window_mode
                );

                player.update_position(&world, deltatime);

                // update player altitude
                player.update_alt(&world);

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
            }
        }
        window.swap_buffers();
        glfw.poll_events();

        // hang thread for target FPS
        while (instant.elapsed().as_millis() as f32) < (1000.0 / target_fps) {}
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>, mouse_captured: &mut bool, selected_coords: &Option<((i32, i32, i32), Option<Face>)>, world: &mut World, player: &mut Player, last_x: &mut f32, last_y: &mut f32, first_mouse: &mut bool, force_recalculation: &mut bool, current_block_index: &mut u32, window_mode: &mut WindowMode) {
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
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                *window_mode = WindowMode::Title;
                window.set_cursor_mode(CursorMode::Normal);
            },
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