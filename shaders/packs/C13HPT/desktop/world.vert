#version 120

varying vec4 col;
varying vec2 uv;
varying float dist;
uniform float wind_time;

void main() {
    vec4 position = gl_Vertex;
    if (gl_Color.a < 0.0) {
        float bend = clamp(-gl_Color.a - 1.0, 0.0, 1.0);
        position.x += sin(wind_time + position.x * 0.17 + position.z * 0.11)
                    * 0.18 * bend;
    }

    vec4 view_position = gl_ModelViewMatrix * position;
    gl_Position = gl_ProjectionMatrix * view_position;
    col = gl_Color;
    uv = gl_MultiTexCoord0.xy;
    dist = length(view_position.xyz);
}
