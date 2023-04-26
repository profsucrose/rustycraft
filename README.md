# rustycraft

Simple Minecraft/Voxal Engine clone written in Rust and OpenGL

<img src="https://i.imgur.com/mIMRoGg.png">

<img src="https://i.imgur.com/8Z2x4CV.png">

## Usage

To run, simply clone the repo and run `cargo run --release`:
```
git clone https://github.com/profsucrose/rustycraft && cd rustycraft && cargo run --release
```

## Local Worlds
To play, you can do so locally by going to "Open World" and typing in a world name (which refers to the name of a world folder in the `game_data/worlds` directory) and then clicking "Open." 

You can move around with `WASD` and jump with `Space`. You can right-click to place a selected block at wherever your cursor is pointing and left-click to destroy any targeted block. You can use the `UpArrow` and `DownArrow` to cycle through the block options. 

The left `Super`/`Command`/`Windows` key can be used to unfocus or focus the window if the cursor is captured. `Esc` is used to exit the current world or server; world chunks are saved automatically whenever a block is placed or destroyed so there's no need to do so manually. 

The `F1` and `F3` keys are used for toggling the GUI/text and for toggling FPS/Fly camera modes respectively.

## Servers

You can host a server from the corresponding repository [here](https://github.com/profsucrose/rustycraft-server)!

To play on a server, go to the "Connect to Server" menu, type in the server address and then click "Connect." Assuming the specified server address is online you will proceed to join and receive chunk data. You can specify a port with `:<port_number>`, but the default is 25566 (which is also the same when hosting a server).

I myself am hosting a little RustyCraft server at `craft.profsucrose.dev`. Feel free to hop on if you want to try out this project's server functionality or potentially try to build something collectively!

### Chat
The functionality/"gameplay" is of course identical to playing locally, however you can send messages in the server chat by pressing `T`, typing your message and pressing `Return` to send it.
