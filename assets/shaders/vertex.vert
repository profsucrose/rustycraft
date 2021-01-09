#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in float aBlockIndex;
// layout (location = 1) in vec2 aTexCoord; 
// layout (location = 2) in float aBlockIndex; 
  
// out vec2 TexCoord;
// out vec3 position;

out VS_OUT {
    float blockIndex;
} vs_out;

void main() {
    gl_Position = vec4(aPos, 1.0);
    // TexCoord = aTexCoord;
    vs_out.blockIndex = aBlockIndex;
    // position = aPos;
}    