#version 120
varying vec4 col;
varying vec2 uv;
varying float dist;
varying float sunSide;
uniform float wind_time;
void main() {
    vec4 position = gl_Vertex;
    if (gl_Color.a < 0.0) {
        float bend = clamp(-gl_Color.a - 1.0, 0.0, 1.0);
        position.x += sin(wind_time + position.x * 0.17 + position.z * 0.11) * 0.18 * bend;
    }
    vec4 viewPos = gl_ModelViewMatrix * position;
    gl_Position = gl_ProjectionMatrix * viewPos;
    col = gl_Color;
    uv = gl_MultiTexCoord0.xy;
    dist = length(viewPos.xyz);
    // Uses the conventional fixed-function normal if supplied by the mesh.
    vec3 n = normalize(gl_NormalMatrix * gl_Normal);
    vec3 sun = normalize(vec3(-0.6, 0.35, 0.7));
    sunSide = clamp(dot(n, sun) * 0.5 + 0.5, 0.0, 1.0);
}
