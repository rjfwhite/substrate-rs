#version 330
out vec4 color;
uniform vec3 paint;
in vec3 v_normal;

void main() {
    float brightness = dot(normalize(v_normal), normalize(vec3(0.4, 1.0, 0.7)));
    vec3 dark_color = paint * 0.4;
    color = vec4(mix(dark_color, paint, brightness), 1.0);
}