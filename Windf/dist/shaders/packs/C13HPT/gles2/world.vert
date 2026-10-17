#version 100
precision highp float;

attribute vec3 position;
attribute vec4 color;
attribute vec2 texcoord;
uniform mat4 model_view;
uniform mat4 projection;
uniform float wind_time;

varying mediump vec4 col;
varying mediump vec2 uv;
varying mediump float dist;

void main() {
    vec3 moved_position = position;
    if (color.a < 0.0) {
        float bend = clamp(-color.a - 1.0, 0.0, 1.0);
        moved_position.x += sin(wind_time + position.x * 0.17 + position.z * 0.11)
                          * 0.18 * bend;
    }

    vec4 view_position = model_view * vec4(moved_position, 1.0);
    gl_Position = projection * view_position;
    col = color;
    uv = texcoord;
    dist = length(view_position.xyz);
}
