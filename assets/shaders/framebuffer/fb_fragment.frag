#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D screenTexture;

void main() {
    FragColor = texture(screenTexture, TexCoords) * vec4(0.2, 0.2, 0.9, 1.0);
}