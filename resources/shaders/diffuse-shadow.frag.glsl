#version 330 core

uniform sampler2DShadow shadow_map;
uniform vec3 light_loc;
uniform vec3 paint;

in vec4 shadow_coord;
in vec4 model_normal;

out vec4 color;

void main() {
    vec3 light_color = vec3(1,1,1);
    float bias = 0.00; // Geometry does not require bias

    float lum = max(dot(normalize(model_normal.xyz), normalize(light_loc)), 0.0);

    float visibility = 1.0;

    if(shadow_coord.y > 0 && shadow_coord.y < 1.0 && shadow_coord.x > 0 && shadow_coord.x < 1.0 && shadow_coord.z < 1.0 && shadow_coord.z > 0.0) {
        //visibility = texture(shadow_map, vec3(shadow_coord.xy, (shadow_coord.z-bias)/shadow_coord.w));

        float shadow = 0.0;
        vec2 texelSize = 1.0 / textureSize(shadow_map, 0);
        for(int x = -1; x <= 1; ++x)
        {
            for(int y = -1; y <= 1; ++y)
            {
                vec2 bits = shadow_coord.xy + vec2(x, y) * texelSize;
                float pcfDepth = texture(shadow_map, vec3( bits, (shadow_coord.z-bias)/shadow_coord.w));
                shadow += shadow_coord.z - bias > pcfDepth ? 1.0 : 0.0;
            }
        }
        shadow /= 9.0;
        visibility = 1.0 - shadow;
    }

    color = vec4(max(lum * visibility, 0.05) * paint * light_color, 1.0);
}