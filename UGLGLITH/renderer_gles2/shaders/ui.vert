#version 100
precision highp float;
attribute vec3 position;
attribute vec4 color;
attribute vec2 texcoord;
uniform mat4 projection;
uniform mat4 model_view;
varying mediump vec4 col;
varying mediump vec2 uv;
void main(){gl_Position=projection*model_view*vec4(position,1.);col=color;uv=texcoord;}
