use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::models::multiplayer::server_world::ServerWorld;

#[derive(Clone)]
pub struct ServerState {
    pub world: Arc<Mutex<ServerWorld>>,
    pub player_names: Arc<Mutex<HashMap<String, String>>>,
    pub chat_stack: Arc<Mutex<Vec<String>>>
}

impl ServerState {
    pub fn new(world: Arc<Mutex<ServerWorld>>) -> ServerState {
        ServerState { 
            world, 
            player_names: Arc::new(Mutex::new(HashMap::new())), 
            chat_stack: Arc::new(Mutex::new(Vec::new())) 
        }
    }
}