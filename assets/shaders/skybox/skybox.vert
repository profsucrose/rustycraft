#version 330 core
layout (location = 0) in vec3 a_pos;

out vec3 texcoords;

uniform mat4 projection;
uniform mat4 view;

void main() {
    tex_coords = a_pos;
    gl_Position = projection * view * vec4(aPos, 1.0);
}