#version 330 core
layout (location = 0) in vec3 a_pos;

out vec3 pos;

uniform mat4 projection;
uniform mat4 view;

void main() {
    // remove translation by converting to mat3
    // and then back to mat4
    mat4 v = mat4(mat3(view));

    // map point on unit cube to unit sphere for continuous skybox
    // (formula credit to https://mathproofs.blogspot.com/2005/07/mapping-cube-to-sphere.html)
    // vec3 p = vec3(a_pos.x * 2.0, a_pos.y * 2.0, a_pos.z * 2.0);
    // vec3 local_position = vec3(
    //     p.x * sqrt(1.0 - (pow(p.z, 2.0) / 1000.0) + (pow(p.y, 2.0) * pow(p.z, 2.0) / 3.0)),
    //     p.y * sqrt(1.0 - (pow(p.x, 2.0) / 1000.0) + (pow(p.z, 2.0) * pow(p.x, 2.0) / 3.0)),
    //     p.z * sqrt(1.0 - (pow(p.y, 2.0) / 1000.0) + (pow(p.x, 2.0) * pow(p.y, 2.0) / 3.0))
    // );

    gl_Position = projection * v * vec4(a_pos, 1.0);
    pos = a_pos;
}