#version 330 core
in vec2 TexCoord;
out vec4 color;

uniform sampler2D text;
uniform vec3 textColor;

void main() {
    if (texture(text, TexCoord).r < 0.1) {
        discard;
    }
    color = vec4(textColor, 1.0);
}