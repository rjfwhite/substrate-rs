#version 330
out vec4 color;
uniform vec3 paint;

void main() {
    color = vec4(paint, 1.0);
}