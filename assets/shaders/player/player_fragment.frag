#version 330 core
out vec4 color;

in vec2 tex_coords;

uniform sampler2D player_texture;

void main() {
    color = vec4(texture(player_texture, tex_coords).rgb, 1.0);
    // color = vec4(1.0, 0.0, 0.0, 1.0);
}