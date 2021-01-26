use std::{io::{self, BufRead, BufReader, LineWriter, Read, Write}, net::TcpStream, sync::{Arc, Mutex}, thread};

use cgmath::Vector3;
use serde_json::Result;

use crate::models::{multiplayer::{rc_message::RustyCraftMessage, server_player::ServerPlayer, server_world::ServerWorld}, utils::vector_utils::get_direction_from_mouse_move};

use super::{event::RustyCraftEvent, server_state::ServerState};

// struct that abstracts reading and writing to a server
pub struct ServerConnection {
    reader: BufReader<TcpStream>,
    writer: LineWriter<TcpStream>,
    stream: TcpStream,
    pub address: String
}

impl Clone for ServerConnection {
    fn clone(&self) -> Self {
        let writer = LineWriter::new(self.stream.try_clone().unwrap());
        let reader = BufReader::new(self.stream.try_clone().unwrap());
        ServerConnection { reader, writer, stream: self.stream.try_clone().unwrap(), address: self.address.clone() }
    }
}

impl ServerConnection {
    pub fn new(mut address: String) -> io::Result<ServerConnection> {
        // default port
        if !address.contains(":") {
            address.push_str(":25566");
        }

        let stream = TcpStream::connect(address.clone());
        if stream.is_err() {
            return Err(stream.unwrap_err());
        }

        let stream = stream.unwrap();
        let writer = LineWriter::new(stream.try_clone()?);
        let reader = BufReader::new(stream.try_clone()?);
        Ok(ServerConnection { reader, writer, address, stream })
    }

    fn send(&mut self, message: &str) -> io::Result<()> {
        self.writer.write(&message.as_bytes())?;
        // send breakline to flush writer
        self.writer.write(&[b'\n'])?;
        Ok(())
    }

    pub fn send_message(&mut self, event: RustyCraftMessage) -> io::Result<()> {
        self.send(serde_json::to_string(&event).unwrap().as_str())
    }

    pub fn read(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;
        match bytes_read {
            0 => Ok(None),
            _ => {
                line.pop();
                Ok(Some(line))
            }
        }
    }

    pub fn create_listen_thread(mut self, state: ServerState) {
        thread::spawn(move || {
            loop {
                let result = self.read().unwrap();
                match result {
                    None => break,
                    Some(data) => {
                        let event: Result<RustyCraftEvent> = serde_json::from_str(data.as_str());
                        if event.is_err() {
                            println!("Received invalid event");
                            continue;
                        }

                        match event.unwrap() {
                            RustyCraftEvent { sender: _, message: RustyCraftMessage::ChunkData { chunks } } => {
                                let mut world = state.world.lock().unwrap();
                                for (chunk_x, chunk_z, serialized_chunk) in chunks.into_iter() {
                                    world.insert_serialized_chunk(chunk_x, chunk_z, serialized_chunk);
                                }
                            },
                            RustyCraftEvent { sender: _, message: RustyCraftMessage::SetBlock { world_x, world_y, world_z, block } } => {
                                let mut server_world = state.world.lock().unwrap();
                                server_world.set_block(world_x, world_y, world_z, block);
                                server_world.recalculate_mesh_from_player_perspective();
                            },
                            // set name can only be done once after player joins, so use it to broadcast
                            // join message
                            RustyCraftEvent { sender, message: RustyCraftMessage::PlayerInit { name, x, y, z } } => {
                                state.players.lock().unwrap().insert(sender.clone(), ServerPlayer::new(sender, name.clone(), x, y, z, 0.0, -90.0));
                                state.chat_stack.lock().unwrap().push(format!("{} joined the server", name));
                            },
                            RustyCraftEvent { sender, message: RustyCraftMessage::PlayerPosition { x, y, z } } => {
                                if let Some(player) = state.players.lock().unwrap().get_mut(&sender) {
                                    player.position = Vector3::new(x, y, z);
                                }
                            },
                            RustyCraftEvent { sender, message: RustyCraftMessage::PlayerDirection { yaw, pitch } } => {
                                if let Some(player) = state.players.lock().unwrap().get_mut(&sender) {
                                    player.yaw = yaw;
                                    player.pitch = pitch;
                                }
                            },
                            RustyCraftEvent { sender, message: RustyCraftMessage::ChatMessage { content } } => {
                                let message = match state.players.lock().unwrap().get(&sender) {
                                    Some(player) => format!("<{}> {}", player.name, content),
                                    None => format!("<Unnamed Player> {}", content)
                                };
                                state.chat_stack.lock().unwrap().push(message);
                            },
                            RustyCraftEvent { sender: _, message: RustyCraftMessage::ConnectionData { id, players } } => {
                                *state.client_id.lock().unwrap() = id;
                                for (id, name, x, y, z, yaw, pitch) in players.iter() {
                                    state.players.lock().unwrap().insert(id.clone(), ServerPlayer::new(id.clone(), name.clone(), *x, *y, *z, *pitch, *yaw));
                                }
                            },
                            RustyCraftEvent { sender, message: RustyCraftMessage::Disconnect } => {
                                let mut players = state.players.lock().unwrap();
                                let player = players.get(&sender);
                                // handle if peer never sent SetName packet
                                let message = match player {
                                    Some(player) => format!("{} left the server", player.name),
                                    None => String::from("[Unnamed Player] left the server")
                                };

                                if player.is_some() {
                                    players.remove(&sender);
                                }
                                state.chat_stack.lock().unwrap().push(message);
                            }
                            event => {
                                println!("Received unhandled event: {:?}", event);
                            }
                        }
                    }
                }
            }
        });
    }
}