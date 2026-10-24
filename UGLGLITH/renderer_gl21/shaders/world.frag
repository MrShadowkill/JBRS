#version 120
uniform sampler2D atlas;
uniform float daylight;
uniform float opacity;
uniform float fog_end;
uniform float detail_amount;
uniform int mesh_mode;
uniform vec3 model_tint;
varying vec4 col;
varying vec2 uv;
varying float dist;
uniform vec3 glitch_params;
uniform float glitch_silhouette;
varying vec3 glitch_position;
void main(){
 vec3 base=col.rgb;float emission=0.;
 if(mesh_mode==1){
  base*=texture2D(atlas,uv).rgb;emission=max(col.a,0.0);
  float grain=sin(uv.x*1200.)*sin(uv.y*900.);
  base*=1.+detail_amount*grain*(1.-clamp(dist/160.,0.,1.));
 }
 if(mesh_mode==2 || mesh_mode==6){base*=model_tint;}
 if(mesh_mode==5){base=texture2D(atlas,uv).rgb;}
 if(mesh_mode==4){emission=1.;}
 if(mesh_mode==3){base=vec3(0.015,0.02,0.015);}
 float night=1.-smoothstep(0.16,0.48,clamp(daylight,0.,1.));
 float light=mesh_mode==3?1.:max(0.25+daylight*0.75,emission*night);
 vec3 sky=vec3(0.08+daylight*0.35,0.11+daylight*0.49,0.2+daylight*0.52);
 float fog=clamp((dist-fog_end*0.55)/(fog_end*0.45),0.,0.7);
 vec3 night_tint=mix(vec3(0.72,0.8,1.),vec3(1.),clamp(daylight,0.,1.));
 vec3 lit=base*light*night_tint;
 lit=mix(lit,lit*vec3(1.035,1.01,0.98),detail_amount*4.);
 vec3 result=mix(lit,sky,fog);
 if(glitch_params.x>0.0 && mesh_mode!=3){
  // Camera-relative cells also corrupt geometry already cached in static VBOs.
  vec3 cell=mod(floor(glitch_position/2.5),32.0);
  float epoch=floor(glitch_params.y*2.0);
  float pick=fract(sin(dot(cell,vec3(1.7,5.3,9.1))+glitch_params.z+epoch)*127.0);
  float checker=mod(floor(uv.x*64.0)+floor(uv.y*64.0)+floor(glitch_position.y*5.0),2.0);
  vec3 broken=mix(vec3(0.008,0.0,0.012),vec3(1.0,0.0,0.82),checker);
  if(pick<0.48){result=mix(result,broken,glitch_params.x);}
  result=mix(result,vec3(0.002),glitch_silhouette);
 }
 gl_FragColor=vec4(result,opacity);
}
