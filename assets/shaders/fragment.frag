#version 330 core
out vec4 FragColor;  

in vec2 TexCoord;

uniform sampler2D dirtTexture;
uniform sampler2D grassTexture; 

void main() {
    FragColor = texture(dirtTexture, TexCoord);
}