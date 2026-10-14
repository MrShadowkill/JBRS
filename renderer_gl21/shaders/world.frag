#version 120
uniform sampler2D atlas;
uniform float daylight;
uniform float opacity;
uniform float fog_end;
uniform float detail_amount;
uniform int mesh_mode;
varying vec4 col;
varying vec2 uv;
varying float dist;
void main(){
 vec3 base=col.rgb;float emission=0.;
 if(mesh_mode==1){
  base*=texture2D(atlas,uv).rgb;emission=col.a;
  float grain=sin(uv.x*1200.)*sin(uv.y*900.);
  base*=1.+detail_amount*grain*(1.-clamp(dist/160.,0.,1.));
 }
 float light=max(0.25+daylight*0.75,emission);
 vec3 sky=vec3(0.08+daylight*0.35,0.11+daylight*0.49,0.2+daylight*0.52);
 float fog=clamp((dist-fog_end*0.55)/(fog_end*0.45),0.,1.);
 vec3 lit=base*light;
 lit=mix(lit,lit*vec3(1.035,1.01,0.98),detail_amount*4.);
 gl_FragColor=vec4(mix(lit,sky,fog),opacity);
}
