#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in float aFrontUV;
layout (location = 2) in float aRightUV;
layout (location = 3) in float aBackUV;
layout (location = 4) in float aBottomUV;
layout (location = 5) in float aLeftUV;
layout (location = 6) in float aTopUV;
layout (location = 7) in float aFacesToDraw;
// layout (location = 1) in vec2 aTexCoord; 
// layout (location = 2) in float aBlockIndex; 
  
// out vec2 TexCoord;
// out vec3 position;

out VS_OUT {
    float blockIndex;
    float[6] blockUVIndices;
    int facesToDraw;
} vs_out;

void main() {
    gl_Position = vec4(aPos, 1.0);
    // TexCoord = aTexCoord;
    vs_out.blockIndex = aFrontUV;
    vs_out.blockUVIndices = float[6](aFrontUV, aRightUV, aBackUV, aBottomUV, aLeftUV, aTopUV);
    vs_out.facesToDraw = int(aFacesToDraw);
    // position = aPos;
}    