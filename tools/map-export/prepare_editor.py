"""Adapt the user-supplied editor for the complete map without changing saved geometry."""
import sys
from pathlib import Path
s=Path(sys.argv[1]).read_text()
s=s.replace('v.length>1000000||f.length>1000000','v.length>2000000||f.length>2000000').replace('максимум 1 000 000 вершин и 1 000 000 граней','максимум 2 000 000 вершин и граней')
s=s.replace('<button id="reset">','<label class="option"><input id="mapPreview" type="checkbox" checked>Легкий огляд (дані повні)</label><label class="option"><input id="isolate" type="checkbox">Лише обрана група</label><button id="focusGroup">Наблизити групу</button><button id="reset">')
s=s.replace('function load(data,name){','let fullBounds=null;\nfunction load(data,name){')
s=s.replace('rawVertices=result.raw;modelCenter=result.center;modelSize=result.size;','rawVertices=result.raw;modelCenter=result.center;modelSize=result.size;fullBounds={center:result.center.slice(),size:result.size};')
s=s.replace('if(history.length>30)history.shift();','if(history.length>(rawVertices.length>100000?2:30))history.shift();')
s=s.replace('function reset(){yaw=.65;pitch=-.35;zoom=1;}',"function reset(){if(fullBounds){modelCenter=fullBounds.center.slice();modelSize=fullBounds.size;sync();}yaw=.65;pitch=faces.length>100000?-.9:-.35;zoom=faces.length>100000?2.5:1;} $('focusGroup').onclick=()=>{const b=bounds();if(!b)return;modelCenter=b.center;modelSize=Math.max(...b.size,0.01);sync();zoom=1;$('isolate').checked=true;};")
s=s.replace('function draw(time){','let mapLastDraw=0;\nfunction draw(time){if(faces.length>100000&&time-mapLastDraw<250){requestAnimationFrame(draw);return;}mapLastDraw=time;')
old='const points=vertices.map(rotate),screen=points.map(project),sorted=faces.map((f,index)=>({f,index,z:f.reduce((s,i)=>s+points[i][2],0)/f.length})).sort((a,b)=>b.z-a.z);'
new="""const step=$('mapPreview').checked&&!$('isolate').checked?Math.max(1,Math.ceil(faces.length/20000)):1;
 const indices=[];for(let i=0;i<faces.length;i+=step){const group=materials[i]?.group;if(hiddenGroups.has(group)||($('isolate').checked&&activeGroup&&group!==activeGroup))continue;indices.push(i);}
 const needed=new Set();for(const i of indices)for(const id of faces[i])needed.add(id);
 const points=Object.create(null),screen=Object.create(null);for(const id of needed){points[id]=rotate(vertices[id]);screen[id]=project(points[id]);}
 const chosen=new Set(selectedIds()||[]),sorted=indices.map(index=>({f:faces[index],index,z:faces[index].reduce((s,i)=>s+points[i][2],0)/faces[index].length})).sort((a,b)=>b.z-a.z);"""
assert old in s;s=s.replace(old,new).replace("if(selectedIds()?.includes(f[0]))","if(chosen.has(f[0]))")
Path(sys.argv[2]).write_text(s)
