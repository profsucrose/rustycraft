#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D screenTexture;

void main() {
    //float contrast = 1.0;
    //vec4 result = texture(screenTexture, TexCoords) * (1.0 + contrast) / 1.0;
    FragColor = texture(screenTexture, TexCoords);
}