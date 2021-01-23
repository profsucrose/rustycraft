#version 330 core
layout (points) in;
layout (triangle_strip, max_vertices = 24) out;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 light_space_matrix;

out vec2 TexCoord;
out vec3 Normal;
out vec3 FragPos;
// send frag position in light space
// to fragment shader for shadow map
// calculations
out vec4 frag_pos_light_space;

in VS_OUT {
    float[6] blockUVIndices;
    int facesToDraw;
} gs_in[];  

const vec4 cubeVerts[8] = vec4[8] (
    vec4(-0.5, -0.5, -0.5, 1.0), // LB  0
    vec4(-0.5,  0.5, -0.5, 1.0), // LT  1
    vec4( 0.5, -0.5, -0.5, 1.0), // RB  2
    vec4( 0.5,  0.5, -0.5, 1.0), // RT  3

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

const vec3 cubeNormals[6] = vec3[6] (
    vec3( 0.0,  0.0, -1.0), // front
    vec3( 1.0,  0.0,  0.0), // right
    vec3( 0.0,  0.0,  1.0), // back
    vec3( 0.0, -1.0,  0.0), // bottom
    vec3(-1.0,  0.0,  0.0), // left
    vec3( 0.0,  1.0,  0.0)  // top
);

const vec2 cubeUVs[24] = vec2[24] (
    // front
    vec2(0.0, 1.0),
    vec2(0.0, 0.0),
    vec2(1.0, 1.0),
    vec2(1.0, 0.0),

    // right
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),

    // back
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),

    // bottom
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),

    // left
    vec2(1.0, 0.0),
    vec2(1.0, 1.0),
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),

    // top
    vec2(1.0, 1.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(0.0, 0.0)
);

void emit_vertex(vec4 local_position, vec2 local_uv, int index) {
    vec4 world_position = gl_in[0].gl_Position;
    vec3 position = (world_position + model * local_position).xyz;
    FragPos = position;
    gl_Position = projection * view * vec4(position, 1.0);
    float blockIndex = gs_in[0].blockUVIndices[index];

    TexCoord = local_uv + vec2(float(int(blockIndex) % 6), float(int(blockIndex) / 6));
    Normal = cubeNormals[index];
    frag_pos_light_space = light_space_matrix * vec4(FragPos, 1.0);
    EmitVertex();
}

void build_cube() {
    int faces = gs_in[0].facesToDraw;

    for (int i = 0; i < 6; i++) {
        // unpack bit to check if face is obfuscated 
        // and should be drawn or not
        if ((faces >> (7 - i) & 1) != 1) {
            continue;
        }

        int indices_index = i * 4;
        emit_vertex(cubeVerts[cubeIndices[indices_index]],     cubeUVs[indices_index],     i); 
        emit_vertex(cubeVerts[cubeIndices[indices_index + 1]], cubeUVs[indices_index + 1], i); 
        emit_vertex(cubeVerts[cubeIndices[indices_index + 2]], cubeUVs[indices_index + 2], i); 
        emit_vertex(cubeVerts[cubeIndices[indices_index + 3]], cubeUVs[indices_index + 3], i); 

        EndPrimitive();
    }
}

void main() {    
    build_cube();
}  