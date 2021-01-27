#version 330 core
layout (location = 0) in vec3 a_pos;

out vec3 pos;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main() {
    gl_Position = projection * view * model * vec4(a_pos, 1.0);
    pos = a_pos;
}