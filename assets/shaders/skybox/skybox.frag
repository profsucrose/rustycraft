#version 330 core
out vec4 color;

in vec3 texcoords;

uniform samplerCube skybox;

void main() {
    color = texture(skybox, texcoords);
}