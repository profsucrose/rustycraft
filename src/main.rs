#![allow(unused_imports)]
mod models;      

use std::{ffi::c_void, fs, mem, path::Path, ptr, sync::{Arc, Mutex, mpsc::Receiver}, time::Instant};
use cgmath::{Deg, Matrix3, Matrix4, Point3, Vector3, vec3};
use glfw::{Action, Context, CursorMode, Key, MouseButton, PixelImage, WindowEvent};
use gl::types::*;
use image::{RgbaImage, GenericImage};
use models::{core::{block_type::index_to_block, player::Player}, opengl::{tex_quad::TexQuad}};
use noise::OpenSimplex;
use rand::{Rng, thread_rng};
use crate::models::{core::{block_type::BlockType, face::Face, window_mode::WindowMode, world::World}, multiplayer::{rc_message::RustyCraftMessage, server_connection::ServerConnection, server_state::ServerState, server_world::ServerWorld}, opengl::{button::Button, camera::Camera, cloud::Cloud, depth_framebuffer::{DepthFrameBuffer, SHADOW_HEIGHT, SHADOW_WIDTH}, framebuffer::FrameBuffer, input::Input, player_model::PlayerModel, shader::Shader, text_renderer::{TextJustification, TextRenderer}, texture::Texture, vertex_array::VertexArray, vertex_buffer::VertexBuffer}, traits::game_world::GameWorld, utils::{name_utils::gen_name, num_utils::distance, simplex_utils::sample}};

// settings
const SCR_WIDTH: u32 = 1000;
const SCR_HEIGHT: u32 = 600;

const SERVER_RENDER_DISTANCE: u32 = 10;
const LOCAL_RENDER_DISTANCE: u32 = 20;

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

    // gl: load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // depth buffer
    gl::Enable(gl::DEPTH_TEST);

    // face culling
    //gl::Enable(gl::CULL_FACE);

    // blending
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

    let shader = Shader::new_with_geom("assets/shaders/voxal/vertex.vert", "assets/shaders/voxal/fragment.frag", "assets/shaders/voxal/geometry.geom");

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
    let text_renderer = TextRenderer::new(SCR_WIDTH, SCR_HEIGHT, "assets/font/OldSchoolAdventures.ttf");

    // target fps
    let target_fps = 60.0;

    let mut selected_coords = None;

    let mut force_recalculation = false;

    let mut current_block_index = 0;

    let water_tint_quad = TexQuad::new("assets/textures/water.png", gl::TEXTURE0, true, SCR_WIDTH, SCR_HEIGHT);

    let mut menu_world = World::new_with_seed(10, "menu_world", 0);

    let mut menu_camera = Camera::new(SCR_WIDTH, SCR_HEIGHT, 0.0);
    menu_world.recalculate_mesh_from_perspective(0, 0);
    menu_camera.position.y = menu_world.highest_in_column(0, 0).unwrap() as f32 + 10.0;
    menu_camera.fov = 60.0;
    menu_camera.mouse_callback(0.0, -140.0);

    let mut yellow_text_size: f32 = 0.0;

    // GUI Widgets 
    // ----------
    // buttons
    let button_width = 420.0;
    let button_height = 48.0;
    let button_x = SCR_WIDTH as f32 / 2.0;
    let select_worlds_button = Button::new("Open World", button_x, 280.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);
    let open_world_button = Button::new("Open", button_x, 210.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);
    let connect_button = Button::new("Connect", button_x, 140.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);
    let mut back_button = Button::new("Back", button_x, 60.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);
    let connect_to_server_button = Button::new("Connect to Server", button_x, 210.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT);

    // inputs
    let mut open_world_input = Input::new(button_x, 280.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT, 1.0, TextJustification::Center);
    let mut connect_to_server_input = Input::new(button_x, 210.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT, 1.0, TextJustification::Center);
    let mut server_player_name_input = Input::new(button_x, 280.0, button_width, button_height, SCR_WIDTH, SCR_HEIGHT, 1.0, TextJustification::Center);
    let mut chat_input = Input::new(SCR_WIDTH as f32 / 2.0, 30.0, SCR_WIDTH as f32 + 30.0, button_height / 1.3, SCR_WIDTH, SCR_HEIGHT, 0.8, TextJustification::Left);

    let last_world = fs::read_to_string("game_data/last_world");
    if last_world.is_ok() {
        open_world_input.text = last_world.unwrap();
    }
    
    let last_server = fs::read_to_string("game_data/last_server");
    if last_server.is_ok() {
        connect_to_server_input.text = last_server.unwrap();
    }

    let player_name = fs::read_to_string("game_data/player_name");
    if player_name.is_ok() {
        server_player_name_input.text = player_name.unwrap();
    } else {
        server_player_name_input.text = gen_name();
    }

    // window
    let mut window_mode = WindowMode::Title;

    // placeholder world object
    let mut world = None;
    let mut server_connection = None;
    let mut server_state = None;
    let mut did_just_fail_to_connect = false;
    let mut shift_pressed = false;
    let mut time = 0.01;
    let mut server_chat_opened = false;
    let mut last_position_before_update_packet = Vector3::new(0.0, 0.0, 0.0);
    let mut update_position_packet = Instant::now();

    // player model object
    let player_model = PlayerModel::new("assets/textures/player_skin.png");

    // cloud model
    let cloud = Cloud::new();
    let cloud_simplex = OpenSimplex::new();
    let mut cloud_z_offset = 0.0;

    let mut show_gui = true;

    let subtitle_text = get_subtitle_text();

    // render loop
    while !window.should_close() {
        let deltatime = instant.elapsed().as_millis() as f32;
        instant = Instant::now();
        time += 0.01;

        // bind framebuffer
        //framebuffer.bind();

        cloud_z_offset += 0.01;

        // clear buffers
        gl::ClearColor(29.0 / 255.0, 104.0 / 255.0, 224.0 / 255.0, 1.0);
        //gl::ClearColor(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); 
        gl::Enable(gl::DEPTH_TEST);
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
                            let last_y = SCR_HEIGHT as f32 - last_y;
                            match window_mode {
                                WindowMode::Title => {
                                    if select_worlds_button.is_hovered(last_x, last_y) {
                                        window_mode = WindowMode::OpenWorld;
                                    }
        
                                    if connect_to_server_button.is_hovered(last_x, last_y) {
                                        window_mode = WindowMode::ConnectToServer;
                                    }
                                },
                                WindowMode::OpenWorld => {
                                    open_world_input.update_focus(last_x, last_y);
                                    if back_button.is_hovered(last_x, last_y) {
                                        window_mode = WindowMode::Title;
                                    }

                                    if open_world_button.is_hovered(last_x, last_y) {
                                        let mut world_object = World::new(LOCAL_RENDER_DISTANCE, open_world_input.text.clone().as_str());
                                        world_object.recalculate_mesh_from_perspective(0, 0);
                                        let last_player_pos = fs::read_to_string(format!("game_data/worlds/{}/player_pos", open_world_input.text.clone().as_str()));
                                        if let Ok(player_pos_str) = last_player_pos {
                                            let words: Vec<&str> = player_pos_str.split(" ").collect();
                                            let x = words[0].parse::<f32>();
                                            let y = words[1].parse::<f32>();
                                            let z = words[2].parse::<f32>();

                                            if let Ok(x) = x {
                                                player.camera.position.x = x;
                                            }

                                            if let Ok(y) = y {
                                                player.camera.position.y = y;
                                            }

                                            if let Ok(z) = z {
                                                player.camera.position.z = z;
                                            }
                                        } else {
                                            player.camera.position.y = world_object.highest_in_column(0, 0).unwrap() as f32 + 2.0;
                                        }
                                        
                                        world = Some(world_object);
                                        window_mode = WindowMode::InWorld;
                                        window.set_cursor_mode(CursorMode::Disabled);
                                        current_block_index = 0;
                                        fs::write("game_data/last_world", open_world_input.text.clone())
                                            .expect("Failed to write world input text to file");
                                    }
                                },
                                WindowMode::ConnectToServer => {
                                    connect_to_server_input.update_focus(last_x, last_y);
                                    server_player_name_input.update_focus(last_x, last_y);
                                    if connect_button.is_hovered(last_x, last_y) {
                                        let address = connect_to_server_input.text.clone();
                                        let connection = ServerConnection::new(address.clone());
                                        match connection {
                                            Err(_) => {
                                                did_just_fail_to_connect = true;
                                            },
                                            Ok(connection) => {
                                                let mut world = ServerWorld::new(SERVER_RENDER_DISTANCE, connection.clone());
                                                world.recalculate_mesh_from_perspective(0, 0);
                                                let world = Arc::new(Mutex::new(world));
                                                server_state = Some(ServerState::new(world.clone()));
                                                connection.clone().send_message(RustyCraftMessage::PlayerJoin { name: String::from(server_player_name_input.text.clone()) })
                                                    .expect("Failed to set name on join");
                                                connection.clone().create_listen_thread(server_state.clone().unwrap());
                                                server_connection = Some(connection.clone());
                                                window.set_cursor_mode(CursorMode::Disabled);
                                                window_mode = WindowMode::InServer;
                                                fs::write("game_data/last_server", address.clone())
                                                    .expect("Failed to write world input text to file");
                                                fs::write("game_data/player_name", server_player_name_input.text.clone())
                                                    .expect("Failed to write world input text to file");
                                                did_just_fail_to_connect = false;
                                                current_block_index = 0;
                                            }
                                        }
                                    }

                                    if back_button.is_hovered(last_x, last_y) {
                                        window_mode = WindowMode::Title;
                                        did_just_fail_to_connect = false;
                                    }
                                },
                                _ => ()
                            }
                        },
                        WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
                        WindowEvent::Key(Key::LeftShift, _, Action::Press, _) => shift_pressed = true,
                        WindowEvent::Key(Key::LeftShift, _, Action::Release, _) => shift_pressed = false,
                        WindowEvent::Key(key, _, Action::Press, _) => {
                            match window_mode {
                                WindowMode::OpenWorld => {
                                    open_world_input.type_key(key, shift_pressed, &text_renderer);
                                },
                                WindowMode::ConnectToServer => {
                                    connect_to_server_input.type_key(key, shift_pressed, &text_renderer);
                                    server_player_name_input.type_key(key, shift_pressed, &text_renderer);
                                },
                                _ => ()
                            }
                        },
                        _ => ()
                    }
                }

                // if window mode was changed to InWorld:
                if window_mode == WindowMode::InWorld || window_mode == WindowMode::InServer {
                    continue;
                }
    
                // text_renderer.render_text("Create World", x + 20.0, 200.0, 1.0, Vector3::new(1.0, 0.0, 0.0));
                // button.draw(x, 180.0, x + 200.0, 230.0, 1.0);
    
                yellow_text_size += 0.075;
                
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // shader uniforms
                shader.use_program();
                shader.set_mat4("view", menu_camera.get_view());
                shader.set_mat4("projection", menu_camera.get_projection());
                shader.set_mat4("model", Matrix4::<f32>::from_scale(1.0));
                shader.set_vec3("light_pos", Vector3::new(menu_camera.position.x, menu_camera.position.y - 2.0, menu_camera.position.z));
                shader.set_vec3("view_pos", menu_camera.position);
                shader.set_float("time", time);

                menu_camera.mouse_callback(0.15, 0.0);
    
                // bind texture
                texture_map.bind();
                shader.set_uint("texture_map", 0);
                
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
                let subtitle_size = 0.7 + (yellow_text_size.sin() + 1.0) / 30.0;
                text_renderer.render_text_with_mat(subtitle_text.as_str(), 870.0 - subtitle_size * 100.0, y - 130.0, subtitle_size, Vector3::new(1.0, 1.0, 0.0), Matrix4::<f32>::from_angle_z(Deg(10.0)), TextJustification::Center);
                text_renderer.render_text("RustyCraft", x, y, 3.5, Vector3::new(1.0, 0.0, 0.0), TextJustification::Center);
                text_renderer.render_text("RustyCraft", x + 3.0, y - 3.0, 3.5, Vector3::new(173.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0), TextJustification::Center);
                text_renderer.render_text("v1.0", SCR_WIDTH as f32 - 60.0, 10.0, 0.9, Vector3::new(1.0, 1.0, 1.0), TextJustification::Left);

                let last_y = SCR_HEIGHT as f32 - last_y;
                match window_mode {
                    WindowMode::Title => {
                        select_worlds_button.draw(&text_renderer, last_x, last_y);
                        connect_to_server_button.draw(&text_renderer, last_x, last_y);
                    },
                    WindowMode::OpenWorld => {
                        back_button.set_y(140.0);
                        open_world_button.draw(&text_renderer, last_x, last_y);
                        open_world_input.draw(&text_renderer);
                        back_button.draw(&text_renderer, last_x, last_y);
                    },
                    WindowMode::ConnectToServer => {
                        back_button.set_y(70.0);
                        connect_button.draw(&text_renderer, last_x, last_y);
                        connect_to_server_input.draw(&text_renderer);
                        server_player_name_input.draw(&text_renderer);
                        back_button.draw(&text_renderer, last_x, last_y);
                        if did_just_fail_to_connect {
                            text_renderer.render_text("Failed to Connect", button_x - 210.0, 310.0, 1.0, Vector3::new(1.0, 1.0, 1.0), TextJustification::Left);
                        }
                    },
                    _ => panic!("Attempted to display window mode that was not a title menu")
                };
            },
            WindowMode::InWorld => {
                let mut world = world.as_mut().unwrap();

                // events
                process_events(
                    &mut window, 
                    &events, 
                    &mut show_gui,
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

                player.update_position(world, deltatime);

                // update player altitude
                player.update_alt(world);

                // draw clouds
                render_clouds(&cloud, &player.camera, &player.camera.position, cloud_z_offset, cloud_simplex);

                if show_gui {
                    // draw text
                    text_renderer.render_text(format!("FPS: {}", (1000.0 / deltatime).round()).as_str(), 10.0, (SCR_HEIGHT as f32) - 30.0, 1.0, vec3(1.0, 1.0, 1.0), TextJustification::Left);
                    text_renderer.render_text(format!("x: {:.2}", player.camera.position.x).as_str(), 10.0, (SCR_HEIGHT as f32) - 50.0, 0.6, vec3(1.0, 1.0, 1.0), TextJustification::Left);
                    text_renderer.render_text(format!("y: {:.2}", player.camera.position.y).as_str(), 10.0, (SCR_HEIGHT as f32) - 70.0, 0.6, vec3(1.0, 1.0, 1.0), TextJustification::Left);
                    text_renderer.render_text(format!("z: {:.2}", player.camera.position.z).as_str(), 10.0, (SCR_HEIGHT as f32) - 90.0, 0.6, vec3(1.0, 1.0, 1.0), TextJustification::Left);

                    let block = index_to_block(current_block_index).unwrap(); 
                    text_renderer.render_text(format!("Selected block: {:?}", block).as_str(), 10.0, (SCR_HEIGHT as f32) - 110.0, 0.6, vec3(1.0, 1.0, 1.0), TextJustification::Left);
                }
                
                // shader uniforms
                shader.use_program();
                // transforms
                shader.set_mat4("view", player.camera.get_view());
                shader.set_mat4("projection", player.camera.get_projection());
                shader.set_mat4("model", Matrix4::<f32>::from_scale(1.0));
                shader.set_vec3("light_pos", player.camera.position);
                shader.set_float("time", time);

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
                    if show_gui {
                        draw_block_selector(x, y, z, face, &shader, &vbo);
                    }
                }

                // couldn't get framebuffer to work for post-processing
                // so draw a blue textured transparent quad for underwater
                // effect now
                if player.underwater(world) {
                    player.camera.speed = 0.003;
                    water_tint_quad.draw(0.0, 0.0, SCR_WIDTH as f32, SCR_HEIGHT as f32, 0.7);
                } else {
                    if player.camera.speed < 0.008 {
                        player.camera.speed = 0.008;
                    }
                }
            },
            WindowMode::InServer => {
                // assume server connection must be Some
                let connection = server_connection.as_mut().unwrap();
                let state = server_state.clone().unwrap();
                let server_world = state.world;

                for (_, event) in glfw::flush_messages(&events) {
                    match event {
                        WindowEvent::FramebufferSize(width, height) => {
                            gl::Viewport(0, 0, width, height);
                        },
                        WindowEvent::Scroll(_, y_offset) => {
                            player.camera.scroll_callback(y_offset as f32);
                        },
                        WindowEvent::CursorPos(xpos, ypos) => {
                            let (x_pos, y_pos) = (xpos as f32, ypos as f32);
                            let x_offset = x_pos - last_x;
                            let y_offset = last_y - y_pos;
                            last_x = x_pos;
                            last_y = y_pos;
            
                            if mouse_captured {
                                player.camera.mouse_callback(x_offset, y_offset);
                                connection.send_message(RustyCraftMessage::PlayerDirection {
                                    yaw: player.camera.yaw,
                                    pitch: player.camera.pitch
                                }).expect("Failed to send movement packet"); 
                            }
                        },
                        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                            if mouse_captured {
                                if let Some(((x, y, z), _)) = selected_coords {
                                    connection.send_message(RustyCraftMessage::SetBlock { world_x: x, world_y: y, world_z: z, block: BlockType::Air })
                                        .expect("Failed to send SetBlock packets");
                                }
                            }
                        },
                        WindowEvent::MouseButton(MouseButton::Button2, Action::Press, _) => {
                            if mouse_captured {
                                if let Some(((x, y, z), Some(face))) = selected_coords {
                                    let place_position = get_block_on_face(x, y, z, &face);
                                    if can_place_block_at_loc(player.camera.position, place_position.0, place_position.1, place_position.2) {
                                        let block = index_to_block(current_block_index);
                                        connection.send_message(RustyCraftMessage::SetBlock { world_x: place_position.0, world_y: place_position.1, world_z: place_position.2, block: block.unwrap() })
                                            .expect("Failed to send SetBlock packets");
                                    }
                                }
                            }
                        }, 
                        WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                            if server_chat_opened {
                                server_chat_opened = false;
                                shift_pressed = false;
                                mouse_captured = true;
                                window.set_cursor_mode(CursorMode::Disabled);
                                chat_input.set_focus(false);
                            } else {
                                window_mode = WindowMode::Title;
                                window.set_cursor_mode(CursorMode::Normal);
                                connection.send_message(RustyCraftMessage::Disconnect)
                                    .expect("Failed to send disconnect message");
                            }
                        },
                        WindowEvent::Key(Key::F3, _, Action::Press, _) => player.toggle_camera(),
                        WindowEvent::Key(Key::F1, _, Action::Press, _) => show_gui = !show_gui,
                        WindowEvent::Key(Key::LeftShift, _, Action::Press, _) if server_chat_opened => shift_pressed = true,
                        WindowEvent::Key(Key::LeftShift, _, Action::Release, _) if server_chat_opened => shift_pressed = false,
                        WindowEvent::Key(Key::Enter, _, Action::Press, _) if server_chat_opened => {
                            connection.send_message(RustyCraftMessage::ChatMessage { content: chat_input.text.clone() })
                                .expect("Failed to send chat message");
                            chat_input.text = String::new();
                            shift_pressed = false;
                            server_chat_opened = false;
                            window.set_cursor_mode(CursorMode::Disabled);
                            mouse_captured = true;
                        },
                        WindowEvent::Key(key, _, Action::Press, _) if server_chat_opened => {
                            chat_input.type_key(key, shift_pressed, &text_renderer);
                        },
                        WindowEvent::Key(Key::T, _, Action::Press, _) => {
                            mouse_captured = false;
                            server_chat_opened = true;
                            window.set_cursor_mode(CursorMode::Normal);
                            chat_input.set_focus(true);
                        },
                        WindowEvent::Key(Key::LeftSuper, _, Action::Press, _) => {
                            mouse_captured = !mouse_captured;
                            window.set_cursor_mode(match mouse_captured {
                                true => CursorMode::Disabled,
                                false => CursorMode::Normal
                            });
                        },
                        WindowEvent::Key(Key::Up, _, Action::Press, _) => {
                            if index_to_block(current_block_index + 1).is_some() {
                                current_block_index += 1
                            }
                        },
                        WindowEvent::Key(Key::Down, _, Action::Press, _) => {
                            if current_block_index > 0 { 
                                current_block_index -= 1 
                            }
                        },
                        WindowEvent::Key(Key::Space, _, Action::Press, _) => player.jump(),
                        WindowEvent::Key(Key::LeftShift, _, Action::Press, _) => player.camera.speed = 0.05,
                        WindowEvent::Key(Key::LeftShift, _, Action::Release, _) => player.camera.speed = 0.008,
                        WindowEvent::Key(key, _, action, _) => player.camera.process_keyboard(key, action),
                        _ => ()
                    }
                }

                // continue if window mode was changed
                if window_mode == WindowMode::Title {
                    continue;
                }
                let position = player.camera.position;

                player.update_position(&*server_world.lock().unwrap(), deltatime);

                // update player altitude
                player.update_alt(&*server_world.lock().unwrap());

                // draw clouds
                render_clouds(&cloud, &player.camera, &player.camera.position, cloud_z_offset, cloud_simplex);

                if show_gui {
                    // draw text
                    text_renderer.render_text(format!("Connected to {}", connection.address).as_str(), 10.0, (SCR_HEIGHT as f32) - 30.0, 1.0, vec3(1.0, 1.0, 1.0), TextJustification::Left);
                    text_renderer.render_text(format!("FPS: {}", (1000.0 / deltatime).round()).as_str(), 10.0, (SCR_HEIGHT as f32) - 60.0, 1.0, vec3(1.0, 1.0, 1.0), TextJustification::Left);
                    text_renderer.render_text(format!("x: {:.2}", player.camera.position.x).as_str(), 10.0, (SCR_HEIGHT as f32) - 80.0, 0.6, vec3(1.0, 1.0, 1.0), TextJustification::Left);
                    text_renderer.render_text(format!("y: {:.2}", player.camera.position.y).as_str(), 10.0, (SCR_HEIGHT as f32) - 100.0, 0.6, vec3(1.0, 1.0, 1.0), TextJustification::Left);
                    text_renderer.render_text(format!("z: {:.2}", player.camera.position.z).as_str(), 10.0, (SCR_HEIGHT as f32) - 120.0, 0.6, vec3(1.0, 1.0, 1.0), TextJustification::Left);

                    let block = index_to_block(current_block_index).unwrap(); 
                    text_renderer.render_text(format!("Selected block: {:?}", block).as_str(), 10.0, (SCR_HEIGHT as f32) - 140.0, 0.6, vec3(1.0, 1.0, 1.0), TextJustification::Left);



                    // chat
                    let chat = state.chat_stack.lock().unwrap();
                    for i in 0..chat.len().min(10) {
                        let message = chat[chat.len() - 1 - i].as_str();
                        text_renderer.render_text(message, 10.0, (i as f32) * 20.0 + 60.0, 0.65, Vector3::new(1.0, 1.0, 1.0), TextJustification::Left);
                    }
                }

                if server_chat_opened {
                    chat_input.draw(&text_renderer);
                }

                // player models
                let client_id = state.client_id.lock().unwrap().clone();
                for (_, p) in state.players.lock().unwrap().iter() {
                    if p.id != client_id {
                        player_model.draw(&player.camera, p.position, p.pitch, p.yaw);
                    }
                }

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

                let x = position.x.round() as i32;
                let z = position.z.round() as i32;
 
                let mut server_world = server_world.lock().unwrap();
                let meshes = server_world.get_world_mesh_from_perspective(x, z, force_recalculation);
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

                selected_coords = server_world.raymarch_block(&player.camera.position, &player.camera.front);
                if let Some(((x, y, z), Some(face))) = selected_coords {
                    if show_gui {
                        draw_block_selector(x, y, z, face, &shader, &vbo);
                    }
                } 

                // couldn't get framebuffer to work for post-processing
                // so draw a blue textured transparent quad for underwater
                // effect now
                if player.underwater(&*server_world) {
                    player.camera.speed = 0.003;
                    water_tint_quad.draw(0.0, 0.0, SCR_WIDTH as f32, SCR_HEIGHT as f32, 0.7);
                } else {
                    if player.camera.speed < 0.008 {
                        player.camera.speed = 0.008;
                    }
                } 

                // send position update packet at 20FPS if position changed
                if (update_position_packet.elapsed().as_millis() as f32) > (1000.0 / 20.0) {
                    if player.camera.position != last_position_before_update_packet {
                        let position = player.camera.position;
                        connection.send_message(RustyCraftMessage::PlayerPosition { 
                            x: position.x, 
                            y: position.y, 
                            z: position.z
                        }).expect("Failed to send movement packet");
                        last_position_before_update_packet = player.camera.position;
                    }
                    update_position_packet = Instant::now();
                }
            }
        }
        
        // second pass, draw framebuffer quad
        // FrameBuffer::unbind();
        // gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        // gl::Clear(gl::COLOR_BUFFER_BIT);
        // framebuffer.draw();

        window.swap_buffers();
        glfw.poll_events();

        // hang thread for target FPS
        while (instant.elapsed().as_millis() as f32) < (1000.0 / target_fps) {}
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>, show_gui: &mut bool, mouse_captured: &mut bool, selected_coords: &Option<((i32, i32, i32), Option<Face>)>, world: &mut World, player: &mut Player, last_x: &mut f32, last_y: &mut f32, first_mouse: &mut bool, force_recalculation: &mut bool, current_block_index: &mut usize, window_mode: &mut WindowMode) {
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
            WindowEvent::Key(Key::F3, _, Action::Press, _) => player.toggle_camera(),
            WindowEvent::Key(Key::F1, _, Action::Press, _) => *show_gui = !*show_gui,
            // WindowEvent::Key(Key::F2, _, Action::Press, _) => {
            //     let width = SCR_WIDTH;
            //     let height = SCR_HEIGHT;
            //     let mut data = vec![0u8; (width * height * 4) as usize].into_boxed_slice();
            //     unsafe { gl::ReadPixels(0, height as i32, width as i32, height as i32, gl::RGBA, gl::UNSIGNED_BYTE, data.as_mut_ptr() as *mut c_void); }

            //     let image = RgbaImage::from_raw(width, height, data.to_vec())
            //         .expect("Unable to convert pixel array to RgbImage");
            //     image.save("screenshot.png").expect("Unable to write image to file");
            //     println!("Saved screenshot");
            // },
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
                    
                    let place_position = get_block_on_face(x, y, z, face);
                    let block = index_to_block(*current_block_index);

                    if !can_place_block_at_loc(player.camera.position, place_position.0, place_position.1, place_position.2) {
                        return;
                    }
                    world.set_block(place_position.0, place_position.1, place_position.2, block.unwrap());
                    *force_recalculation = true;
                }
            }, 
            WindowEvent::Key(Key::Space, _, Action::Press, _) => player.jump(),
            WindowEvent::Key(Key::Up, _, Action::Press, _) => {
                if index_to_block(*current_block_index + 1).is_some() {
                    *current_block_index += 1
                }
            },
            WindowEvent::Key(Key::Down, _, Action::Press, _) => if *current_block_index > 0 { *current_block_index -= 1 },
            WindowEvent::Key(Key::LeftShift, _, Action::Press, _) => player.camera.speed = 0.05,
            WindowEvent::Key(Key::LeftShift, _, Action::Release, _) => player.camera.speed = 0.008,
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                *current_block_index = 0;
                *window_mode = WindowMode::Title;
                window.set_cursor_mode(CursorMode::Normal);
                fs::write(format!("{}/player_pos", world.save_dir).as_str(), format!("{} {} {}", player.camera.position.x, player.camera.position.y, player.camera.position.z))
                    .expect("Failed to write player position to file");
            },
            WindowEvent::Key(Key::LeftSuper, _, Action::Press, _) => {
                *mouse_captured = !*mouse_captured;
                window.set_cursor_mode(match *mouse_captured {
                    true => CursorMode::Disabled,
                    false => CursorMode::Normal
                });
            },
            WindowEvent::Key(key, _, action, _) => player.camera.process_keyboard(key, action),
            _ => ()
        }
    }
}

unsafe fn draw_block_selector(x: i32, y: i32, z: i32, face: Face, shader: &Shader, vbo: &VertexBuffer) {
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

fn can_place_block_at_loc(player_position: Vector3<f32>, x: i32, y: i32, z: i32) -> bool {
    x != player_position.x.round() as i32
        || (y != player_position.y.round() as i32
        && y != player_position.y.round() as i32 - 1)
        || z != player_position.z.round() as i32
}

fn get_block_on_face(x: i32, y: i32, z: i32, face: &Face) -> (i32, i32, i32) {
    match face {
        Face::Top => (x, y + 1, z),
        Face::Bottom => (x, y - 1, z),
        Face::Right => (x + 1, y, z),
        Face::Left => (x - 1, y, z),
        Face::Front => (x, y, z - 1),
        Face::Back => (x, y, z + 1),
    }
}

fn get_subtitle_text() -> String {
    let mut rng = rand::thread_rng();
    let s = match rng.gen_range(0..8) {
        0 => "Bad Minecraft -- in Rust!",
        1 => "Extra mutable aliasing!",
        2 => "Unperformant, unreliable & unproductive!",
        3 => "unsafe { Game::minecraft() }",
        4 => "Make boredom subtype 'static!",
        5 => "None.unwrap()",
        6 => "The No. 1 Craft!",
        _ => "Golang bad!"
    };
    String::from(s)
}

unsafe fn render_clouds(cloud: &Cloud, camera: &Camera, player_position: &Vector3<f32>, z_offset: f32, simplex: OpenSimplex) {
    let cloud_x = (player_position.x as i32) / 10;
    let cloud_z = (player_position.z as i32) / 10;
    for x in 0..LOCAL_RENDER_DISTANCE * 4 {
        for z in 0..LOCAL_RENDER_DISTANCE * 4 {
            let local_x = LOCAL_RENDER_DISTANCE as i32 * 2 - x as i32;
            let local_z = LOCAL_RENDER_DISTANCE as i32 * 2 - z as i32;
            let x = LOCAL_RENDER_DISTANCE as i32 * 2 - x as i32 + cloud_x;
            let z = LOCAL_RENDER_DISTANCE as i32 * 2 - z as i32 + cloud_z - z_offset as i32 / 10;
            if distance(0, 0, local_x, local_z) < 2.0 * LOCAL_RENDER_DISTANCE as f32 {
                let noise = sample(x as f32 / 4.0, z as f32 / 4.0, simplex);
                if noise > 0.5 {
                    cloud.draw(camera, x as i32, 100, z as i32, z_offset);
                }
            }
        } 
    }
}