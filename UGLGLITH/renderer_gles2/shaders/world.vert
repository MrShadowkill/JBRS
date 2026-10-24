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
uniform mediump vec3 glitch_params;
varying mediump vec3 glitch_position;
void main(){ vec3 p=position; if(color.a<0.0){float bend=clamp(-color.a-1.0,0.0,1.0);p.x+=sin(wind_time+position.x*0.17+position.z*0.11)*0.18*bend;} vec4 v=model_view*vec4(p,1.); glitch_position=v.xyz; if(glitch_params.x>0.0){float band=floor(v.y*3.0);float jump=sin(band*13.7+floor(glitch_params.y*13.0)+glitch_params.z);v.x+=step(0.55,jump)*sin(band+glitch_params.y*19.0)*0.65*glitch_params.x;} gl_Position=projection*v; col=color; uv=texcoord; dist=length(v.xyz); }
