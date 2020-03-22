#version 330 core
in vec3 position;
in mat4 model;
uniform mat4 projection;
uniform mat4 view;
void main() {
    gl_Position = projection * view * model * vec4(position, 1);
}