#version 330 core
out vec4 FragColor;  

in vec2 TexCoord;
in vec3 Normal;
in vec3 FragPos;
in vec4 frag_pos_light_space;

uniform sampler2D texture_map;
uniform sampler2D shadow_map;

uniform vec3 light_pos;
uniform vec3 view_pos;
uniform float time;

/*
float shadow_calculation(vec4 frag_pos_light_space) {
    // perform perspective divide 
    vec3 proj_coords = frag_pos_light_space.xyz / frag_pos_light_space.w;

    // transform to 0-1 uv coord range for sampling
    proj_coords = proj_coords * 0.5 + 0.5;

    float closest_depth = texture(shadow_map, proj_coords.xy).r;
    float current_depth = proj_coords.z;

    // if fragment z is behind depth map z than it's in shadow
    float bias = 0.05;
    float shadow = (current_depth + bias) > closest_depth ? 1.0 : 0.0;
    return shadow;
}
*/

void main() {
    vec4 color = texture(texture_map, vec2(TexCoord.x / 6.0, TexCoord.y / 6.0));

    // blinn-phong lighting
    vec3 norm = normalize(Normal); 
    vec3 light_color = vec3(1.0, 1.0, 1.0);

    // ambient
    float ambient_strength = 0.5;
    vec3 ambient = ambient_strength * light_color;

    // diffuse
    vec3 light_dir = vec3(-0.8, -1.0, 0.0);
    light_dir = normalize(-light_dir);
    float diff = max(dot(light_dir, norm), 0.0);
    vec3 diffuse = 0.8 * diff * light_color;

    // specular
    vec3 view_dir = normalize(view_pos - FragPos);
    vec3 halfway_dir = normalize(light_dir + view_dir);
    float spec = pow(max(dot(norm, halfway_dir), 0.0), 10);
    vec3 specular = spec * light_color;

    // shadow
    // float shadow = shadow_calculation(frag_pos_light_space);

    // calculate result by summing light sources
    vec3 lighting = (ambient + diffuse /*+ specular*/) * color.rgb;
    FragColor = vec4(lighting, color.a);
}