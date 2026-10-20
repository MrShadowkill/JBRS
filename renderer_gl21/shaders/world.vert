#version 120
varying vec4 col; varying vec2 uv; varying float dist; varying vec3 view_pos; uniform float wind_time; void main(){vec4 v=gl_ModelViewMatrix*gl_Vertex;if(gl_Color.a<0.0){float bend=clamp(-gl_Color.a-1.0,0.0,1.0);v.x+=sin(wind_time+gl_Vertex.x*0.17+gl_Vertex.z*0.11)*0.18*bend;} gl_Position=gl_ProjectionMatrix*v; col=gl_Color;uv=gl_MultiTexCoord0.xy;view_pos=v.xyz;dist=length(v.xyz);}
