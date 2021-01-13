#version 330 core
out vec4 FragColor;  

in vec2 TexCoord;

uniform sampler2D texture_map;

void main() {
    FragColor = texture(texture_map, vec2(TexCoord.x / 6.0, TexCoord.y / 6.0));
    //FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    if (FragColor.a < 0.1)
        discard;
}