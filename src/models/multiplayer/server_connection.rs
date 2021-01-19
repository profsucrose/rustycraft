use std::{io::{self, BufRead, BufReader, LineWriter, Read, Write}, net::TcpStream, sync::{Arc, Mutex}, thread};

use serde_json::Result;

use crate::models::multiplayer::{rc_message::RustyCraftMessage, server_world::ServerWorld};

use super::event::RustyCraftEvent;

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
    pub fn new(address: String) -> io::Result<ServerConnection> {
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

    pub fn create_listen_thread(mut self, server_world: Arc<Mutex<ServerWorld>>) {
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
                                let mut world = server_world.lock().unwrap();
                                println!("Received {} chunks", chunks.len());
                                for (chunk_x, chunk_z, serialized_chunk) in chunks.into_iter() {
                                    world.insert_serialized_chunk(chunk_x, chunk_z, serialized_chunk);
                                }
                            },
                            RustyCraftEvent { sender: _, message: RustyCraftMessage::SetBlock { world_x, world_y, world_z, block } } => {
                                println!("{}", data);
                                let mut server_world = server_world.lock().unwrap();
                                server_world.set_block(world_x, world_y, world_z, block);
                                server_world.recalculate_mesh_from_player_perspective();
                            },
                            _ => ()
                        }
                    }
                }
            }
            println!("Stopped read thread");
        });
    }
}