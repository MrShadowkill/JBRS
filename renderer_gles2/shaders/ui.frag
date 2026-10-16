#version 100
precision mediump float;
uniform sampler2D atlas;
uniform int textured;
varying mediump vec4 col;
varying mediump vec2 uv;
void main(){gl_FragColor=col;if(textured==1){gl_FragColor*=texture2D(atlas,uv);}}
