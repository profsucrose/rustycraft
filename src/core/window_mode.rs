// enum representing window state
#[derive(Debug, PartialEq)]
pub enum WindowMode {
    Title,
    OpenWorld,
    ConnectToServer,
    InWorld,
    InServer
}