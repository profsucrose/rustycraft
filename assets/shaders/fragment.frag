#version 330 core
out vec4 FragColor;  

in vec2 TexCoord;

uniform sampler2D texture1;
uniform sampler2D texture2;
  
void main() {
    vec4 color = texture(texture1, TexCoord);
    if(color.a < 0.1)
        discard;
    FragColor = color;
}