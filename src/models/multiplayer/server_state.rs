use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::models::multiplayer::server_world::ServerWorld;

use super::server_player::ServerPlayer;

#[derive(Clone)]
pub struct ServerState {
    pub client_id: Arc<Mutex<String>>,
    pub world: Arc<Mutex<ServerWorld>>,
    pub players: Arc<Mutex<HashMap<String, ServerPlayer>>>,
    pub chat_stack: Arc<Mutex<Vec<String>>>,
}

impl ServerState {
    pub fn new(world: Arc<Mutex<ServerWorld>>) -> ServerState {
        ServerState { 
            client_id: Arc::new(Mutex::new(String::new())),
            world, 
            players: Arc::new(Mutex::new(HashMap::new())), 
            chat_stack: Arc::new(Mutex::new(Vec::new())) 
        }
    }
}