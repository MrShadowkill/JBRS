#version 120
varying vec4 col; varying vec2 uv; varying float dist; uniform float wind_time; uniform vec3 glitch_params;
varying vec3 glitch_position;
void main(){vec4 v=gl_ModelViewMatrix*gl_Vertex;if(gl_Color.a<0.0){float bend=clamp(-gl_Color.a-1.0,0.0,1.0);v.x+=sin(wind_time+gl_Vertex.x*0.17+gl_Vertex.z*0.11)*0.18*bend;} glitch_position=v.xyz; if(glitch_params.x>0.0){float band=floor(v.y*3.0);float jump=sin(band*13.7+floor(glitch_params.y*13.0)+glitch_params.z);v.x+=step(0.55,jump)*sin(band+glitch_params.y*19.0)*0.65*glitch_params.x;} gl_Position=gl_ProjectionMatrix*v; col=gl_Color;uv=gl_MultiTexCoord0.xy;dist=length(v.xyz);}
