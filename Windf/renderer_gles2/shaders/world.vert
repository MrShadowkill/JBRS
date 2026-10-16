#version 100
precision highp float;
attribute vec3 position;
attribute vec4 color;
attribute vec2 texcoord;
uniform mat4 model_view;
uniform mat4 projection;
varying mediump vec4 col;
varying mediump vec2 uv;
varying mediump float dist;
void main(){ vec4 v=model_view*vec4(position,1.); gl_Position=projection*v; col=color; uv=texcoord; dist=length(v.xyz); }
