#version 330 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 depth_bias_mvp;

in vec3 position;
in vec3 normal;

out vec4 shadow_coord;
out vec3 model_normal;

void main() {
    gl_Position =  projection * view * model * vec4(position, 1.0);
    model_normal = mat3(model) * normal;
    shadow_coord = depth_bias_mvp * vec4(position, 1.0);
}