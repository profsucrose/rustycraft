#version 330 core
layout (points) in;
layout (triangle_strip, max_vertices = 24) out;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec2 TexCoord;

in VS_OUT {
    float blockIndex;
} gs_in[];  

// thank you to StackOverflow user for vertex data
const vec4 cubeVerts[8] = vec4[8] (
    vec4(-0.5, -0.5, -0.5, 1.0), //LB  0
    vec4(-0.5,  0.5, -0.5, 1.0), //LT  1
    vec4( 0.5, -0.5, -0.5, 1.0), //RB  2
    vec4( 0.5,  0.5, -0.5, 1.0), //RT  3

    vec4(-0.5, -0.5, 0.5, 1.0), // LB  4
    vec4(-0.5,  0.5, 0.5, 1.0), // LT  5
    vec4( 0.5, -0.5, 0.5, 1.0), // RB  6
    vec4( 0.5,  0.5, 0.5, 1.0)  // RT  7
);

const int cubeIndices[24] = int[24] (
    0, 1, 2, 3, // front
    7, 6, 3, 2, // right
    7, 5, 6, 4, // back
    4, 0, 6, 2, // bottom
    1, 0, 5, 4, // left
    3, 1, 7, 5  // top
); 

void emit_vertex(vec4 local_position, vec2 local_uv) {
    vec4 world_position = gl_in[0].gl_Position;
    gl_Position = projection * view * model * vec4((world_position + local_position).xyz, 1.0);
    float blockIndex = gs_in[0].blockIndex;
    TexCoord = local_uv + vec2(blockIndex, 0.0);
    EmitVertex();
}

void build_cube() {
    for (int i = 0; i < 6; i++) {
        int indices_index = i * 4;
        // emit_vertex(position, cubeVerts[cubeIndices[indices_index]], vec2(0.0, 1.0)); // bottom-left
        // emit_vertex(position, cubeVerts[cubeIndices[indices_index + 1]], vec2(1.0, 1.0)); // bottom-right
        // emit_vertex(position, cubeVerts[cubeIndices[indices_index + 2]], vec2(0.0, 0.0)); // top-left
        // emit_vertex(position, cubeVerts[cubeIndices[indices_index + 3]], vec2(1.0, 0.0)); // top-right
        emit_vertex(cubeVerts[cubeIndices[indices_index]],     vec2(0.0, 1.0)); // bottom-left
        emit_vertex(cubeVerts[cubeIndices[indices_index + 1]], vec2(1.0, 1.0)); // bottom-right
        emit_vertex(cubeVerts[cubeIndices[indices_index + 2]], vec2(0.0, 0.0)); // top-left
        emit_vertex(cubeVerts[cubeIndices[indices_index + 3]], vec2(1.0, 0.0)); // top-right

        EndPrimitive();
    }

    // emit_vertex(position, vec4(-0.2, -0.2, 1.0, 0.0));
    // emit_vertex(position, vec4( 0.2, -0.2, 1.0, 0.0));
    // emit_vertex(position, vec4(-0.2,  0.2, 1.0, 0.0));
    // emit_vertex(position, vec4( 0.2,  0.2, 1.0, 0.0));
    // EndPrimitive();
}

void main() {    
    build_cube();
}  