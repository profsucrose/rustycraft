#version 330 core
layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

uniform mat4 orthographic;

void main() {
    gl_Position = orthographic * vec4(aPos, 0.0, 1.0);
    TexCoord = aTexCoord;
}