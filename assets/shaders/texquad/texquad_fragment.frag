#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D tex;
uniform float alpha;

void main() {
    FragColor = vec4(texture(tex, TexCoords).rgb, alpha);
}