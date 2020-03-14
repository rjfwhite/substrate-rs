#version 330
in vec3 position;
in vec3 normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 v_normal;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    v_normal = transpose(inverse(mat3(model))) * normal;
}