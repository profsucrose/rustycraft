use super::rc_message::RustyCraftMessage;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RustyCraftEvent {
    pub sender: String,
    pub message: RustyCraftMessage
}