#version 120

uniform sampler2D atlas;
uniform float daylight;
uniform float opacity;
uniform float fog_end;
uniform float detail_amount;
uniform int mesh_mode;
uniform vec3 model_tint;
varying  vec4 col;
varying  vec2 uv;
varying  float dist;
varying float sunSide;

void main() {
    float day = clamp(daylight, 0.0, 1.0);
    // Golden hour peaks around twilight; normal daytime stays neutral.
    float dusk = smoothstep(0.07, 0.27, day) * (1.0 - smoothstep(0.53, 0.85, day));
    float night = 1.0 - smoothstep(0.08, 0.35, day);

    vec3 base = col.rgb;
    float emission = 0.0;
    if (mesh_mode == 1) {
        base *= texture2D(atlas, uv).rgb;
        emission = max(col.a, 0.0);
        float grain = sin(uv.x * 900.0) * sin(uv.y * 700.0);
        base *= 1.0 + clamp(detail_amount, 0.0, 0.2) * grain *
                (1.0 - clamp(dist / 150.0, 0.0, 1.0));
    }
    if (mesh_mode == 2) base *= model_tint;
    if (mesh_mode == 3) base = vec3(0.015, 0.02, 0.015);
    if (mesh_mode == 4) emission = 1.0;

    vec3 warm = vec3(1.21, 0.94, 0.69);
    vec3 coolShadow = vec3(0.56, 0.68, 0.91);
    vec3 dayLight = vec3(1.0, 1.0, 0.98);
    vec3 ambient = mix(vec3(0.73, 0.82, 1.02), dayLight, day);
    float shade = sunSide;
    // Soft, coloured evening shade; not a true geometry-cast shadow map.
    vec3 duskLight = mix(coolShadow * 0.66, warm * 1.11, shade);
    vec3 lightColor = mix(ambient, duskLight, dusk * 0.88);
    float night_lights = 1.0 - smoothstep(0.16, 0.48, day);
    float brightness = max(0.23 + day * 0.77, emission * night_lights);
    vec3 lit = base * lightColor * brightness;
    if (mesh_mode == 3) lit = base;

    // Gentle filmic contrast, no expensive post-processing pass.
    lit = max(lit, vec3(0.0));
    lit = lit / (vec3(1.0) + lit * 0.13);
    lit *= mix(vec3(1.0), vec3(0.86, 0.91, 1.07), night * 0.23);

    vec3 daySky = vec3(0.43, 0.64, 0.80);
    vec3 duskSky = vec3(0.83, 0.54, 0.37);
    vec3 nightSky = vec3(0.055, 0.08, 0.14);
    vec3 sky = mix(nightSky, daySky, smoothstep(0.05, 0.52, day));
    sky = mix(sky, duskSky, dusk * 0.83);
    float fogRange = max(fog_end, 1.0);
    float fog = smoothstep(fogRange * 0.48, fogRange, dist) * 0.75;
    gl_FragColor = vec4(mix(lit, sky, fog), opacity);
}
