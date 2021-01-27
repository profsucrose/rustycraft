use cgmath::Vector3;

#[derive(Debug)]
pub struct ServerPlayer {
    pub id: String,
    pub name: String,
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32
    //pub skin: Texture
}

impl ServerPlayer {
    pub fn new(id: String, name: String, x: f32, y: f32, z: f32, pitch: f32, yaw: f32) -> ServerPlayer {
        ServerPlayer {
            id,
            name,
            position: Vector3::new(x, y, z),
            rotation: Vector3::new(0.0, 0.0, -1.0),
            pitch, 
            yaw
        }
    }
}