#version 330 core
layout (location = 0) in vec3 position; 
layout (location = 1) in vec2 a_texcoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec2 tex_coords;

void main() {
   tex_coords = a_texcoords;
    gl_Position = projection * view * model * vec4(position, 1.0);
}