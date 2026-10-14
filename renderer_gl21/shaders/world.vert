#version 120
varying vec4 col; varying vec2 uv; varying float dist; void main(){vec4 v=gl_ModelViewMatrix*gl_Vertex; gl_Position=gl_ProjectionMatrix*v; col=gl_Color;uv=gl_MultiTexCoord0.xy;dist=length(v.xyz);}
