#version 100
precision mediump float;

uniform sampler2D atlas;
uniform float daylight;
uniform float opacity;
uniform float fog_end;
uniform float detail_amount;
uniform int mesh_mode;
uniform vec3 model_tint;

varying mediump vec4 col;
varying mediump vec2 uv;
varying mediump float dist;

void main() {
    if (mesh_mode == 3) {
        gl_FragColor = vec4(vec3(0.015, 0.02, 0.015), opacity);
        return;
    }

    float day = clamp(daylight, 0.0, 1.0);
    float twilight = smoothstep(0.05, 0.32, day)
                   * (1.0 - smoothstep(0.48, 0.78, day));
    float night = 1.0 - smoothstep(0.04, 0.34, day);

    vec3 base = col.rgb;
    float emission = 0.0;
    if (mesh_mode == 1) {
        base *= texture2D(atlas, uv).rgb;
        emission = max(col.a, 0.0);
    } else if (mesh_mode == 2) {
        base *= model_tint;
    } else if (mesh_mode == 4) {
        emission = 1.0;
    }

    float night_lights = 1.0 - smoothstep(0.16, 0.48, day);
    float light_level = max(mix(0.28, 1.0, day), emission * night_lights);
    vec3 night_tint = vec3(0.70, 0.79, 1.0);
    vec3 sunlight = mix(night_tint, vec3(1.0, 0.99, 0.95), day);
    vec3 sunset = vec3(1.10, 0.82, 0.62);
    vec3 lit = base * sunlight * light_level;
    lit = mix(lit, lit * sunset, twilight * 0.32 * clamp(detail_amount, 0.0, 1.0));

    float grade = clamp(detail_amount, 0.0, 0.2) * 0.35;
    lit = max(lit, vec3(0.0));
    lit = lit / (vec3(1.0) + lit * grade);
    lit *= mix(vec3(1.0), vec3(0.83, 0.89, 1.06), night * 0.22);

    vec3 day_fog = vec3(0.48, 0.64, 0.74);
    vec3 night_fog = vec3(0.075, 0.10, 0.17);
    vec3 dusk_fog = vec3(0.72, 0.48, 0.36);
    vec3 fog_color = mix(night_fog, day_fog, smoothstep(0.02, 0.48, day));
    fog_color = mix(fog_color, dusk_fog, twilight * 0.42);
    float fog_limit = max(fog_end, 1.0);
    float fog = smoothstep(fog_limit * 0.54, fog_limit, dist) * 0.68;
    gl_FragColor = vec4(mix(lit, fog_color, fog), opacity);
}
