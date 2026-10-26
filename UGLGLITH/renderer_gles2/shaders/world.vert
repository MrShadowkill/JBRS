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
varying mediump vec3 glitch_position;
uniform mediump vec3 glitch_params;
uniform float wind_time;
uniform mediump int mesh_mode;
uniform float chaos_anchor;
uniform vec3 chaos_anchor_position;
uniform float chaos_motion;
uniform float noodle_amount;
void main() {
 vec4 input_color=color; vec3 p=position;
 if(noodle_amount>0.0) {
  p.x+=sin(p.z*3.0+wind_time*3.0)*0.5*noodle_amount;
  p.y+=cos(p.z*2.0+wind_time*3.0)*0.3*noodle_amount;
 }
 if(mesh_mode>=5) {
  if(input_color.a>144.0) {
   p.z+=sin(chaos_motion)*(input_color.a<=180.0?0.3:-0.3);
  }
  if(chaos_anchor>0.0 && abs(input_color.a-chaos_anchor)<0.25) {
   p=chaos_anchor_position;
  }
 }
 if(input_color.a<0.0) {
  float bend=clamp(-input_color.a-1.0,0.0,1.0);
  p.x+=sin(wind_time+p.x*0.17+p.z*0.11)*0.18*bend;
 }
 vec4 v=model_view*vec4(p,1.0);
 glitch_position=v.xyz;
 if(glitch_params.x>0.0 && !(mesh_mode>=5 && chaos_anchor>0.0 && abs(input_color.a-chaos_anchor)<0.25)) {
  float band=floor(v.y*3.0);
  float jump=sin(band*13.7+floor(glitch_params.y*13.0)+glitch_params.z);
  v.x+=step(0.55,jump)*sin(band+glitch_params.y*19.0)*0.65*glitch_params.x;
 }
 gl_Position=projection*v; col=input_color; uv=texcoord; dist=length(v.xyz);
}
