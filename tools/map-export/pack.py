"""Exact triangle/quad export plus independently selectable box primitives (stdlib)."""
import argparse,json,math,struct
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('input',type=Path);p.add_argument('output',type=Path);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True);(a.output/'parts').mkdir(exist_ok=True)
full={k:[] for k in ['vertices','faces','faceMaterials','primitives']};part={k:[] for k in full};parts=[];shared={};local={};triangles=0;building_count=0
box_faces=[[0,1,2,3],[5,4,7,6],[4,0,3,7],[1,5,6,2],[3,2,6,7],[4,5,1,0]]
pattern=[i for face in box_faces for i in [face[0],face[1],face[2],face[0],face[2],face[3]]]
buildings={}
for line in (a.input/'buildings.tsv').read_text().splitlines():
 id,name,x,z=line.split('\t');buildings.setdefault((int(x),int(z)),[]).append((id,name,(a.input/f'building_{id}.bin').read_bytes()))

def save_part():
 global part,shared
 if not part['faces']:return
 name=f'parts/part-{len(parts):03d}.json';path=a.output/name;path.write_text(json.dumps(part,separators=(',',':'),ensure_ascii=False))
 assert len(part['vertices'])<=100000 and len(part['faces'])<=100000 and path.stat().st_size<=15*1024*1024
 parts.append({'file':name,'vertices':len(part['vertices']),'faces':len(part['faces'])});part={k:[] for k in full};shared={}
def color(v):return '#'+''.join(f'{round(max(0,min(1,c))*255):02x}' for c in v[3:6])
def add(points,faces,colors,group,primitive):
 global part,shared
 if len(part['vertices'])+len(points)>90000 or len(part['faces'])+len(faces)>90000:save_part()
 for target,index in [(full,local),(part,shared)]:
  ids=[]
  for point in points:
   key=(stem,group,point)
   if primitive or key not in index:
    j=len(target['vertices']);target['vertices'].append(point)
    if not primitive:index[key]=j
   else:j=index[key]
   ids.append(j)
  start=len(target['faces'])
  for face,c in zip(faces,colors):target['faces'].append([ids[i] for i in face]);target['faceMaterials'].append({'color':c,'group':group})
  if primitive:target['primitives'].append({'type':'box','vertices':ids,'firstFace':start,'faceCount':len(faces),'group':group})

for line in (a.input/'meshes.tsv').read_text().splitlines():
 stem,base,count=line.split('\t');raw=(a.input/(stem+'.bin')).read_bytes();vs=list(struct.iter_unpack('<7f',raw));assert len(vs)==int(count)*3
 assert all(math.isfinite(v) for row in vs for v in row)
 intervals=[]
 if stem.startswith('chunk_'):
  _,x,z=stem.split('_')
  for id,name,data in buildings.get((int(x),int(z)),[]):
   start=raw.find(data);assert start>=0 and start%84==0,(id,name)
   intervals.append((start//28,(start+len(data))//28,f'Buildings/{name} {id}'));building_count+=1
 intervals.sort();cursor=0;local={};i=0
 while i<len(vs):
  while cursor<len(intervals) and i>=intervals[cursor][1]:cursor+=1
  group=intervals[cursor][2] if cursor<len(intervals) and intervals[cursor][0]<=i<intervals[cursor][1] else base
  boundary=intervals[cursor][1] if cursor<len(intervals) and intervals[cursor][0]<=i else (intervals[cursor][0] if cursor<len(intervals) else len(vs))
  available=min(len(vs),boundary)-i
  # Recognize the actual 12-triangle cuboid topology emitted by Mesh::cuboid.
  isbox=False
  if available>=36:
   seq=vs[i:i+36];pts=[seq[j][:3] for j in [0,1,2,5,7,6,11,8]]
   isbox=len(set(pts))==8 and all(seq[j][:3]==pts[k] for j,k in enumerate(pattern))
   if isbox:
    # Reject wedges: box edges must be mutually orthogonal.
    edges=[[pts[j][k]-pts[0][k] for k in range(3)] for j in [1,3,4]]
    isbox=all(abs(sum(x*y for x,y in zip(edges[u],edges[v])))<1e-4*max(1,math.sqrt(sum(x*x for x in edges[u])*sum(x*x for x in edges[v]))) for u,v in [(0,1),(0,2),(1,2)])
  if isbox:
   add(pts,box_faces,[color(seq[j]) for j in range(0,36,6)],group,True);i+=36;triangles+=12;continue
  # A quad keeps exactly the original 0-2 diagonal, including non-planar terrain.
  if available>=6 and vs[i][:3]==vs[i+3][:3] and vs[i+2][:3]==vs[i+4][:3] and all(v[3:6]==vs[i][3:6] for v in vs[i:i+6]):
   add([vs[i+j][:3] for j in [0,1,2,5]],[[0,1,2,3]],[color(vs[i])],group,False);i+=6;triangles+=2
  else:
   add([v[:3] for v in vs[i:i+3]],[[0,1,2]],[color(vs[i])],group,False);i+=3;triangles+=1
 print(stem,flush=True)
save_part();path=a.output/'JBRS-full-map.json';path.write_text(json.dumps(full,separators=(',',':'),ensure_ascii=False))
assert triangles==sum(int(line.split('\t')[2]) for line in (a.input/'meshes.tsv').read_text().splitlines())
manifest={'vertices':len(full['vertices']),'faces':len(full['faces']),'source_triangles':triangles,'box_primitives':len(full['primitives']),'building_exteriors':building_count,'groups':len(set(m['group'] for m in full['faceMaterials'])),'bytes':path.stat().st_size,'parts':parts,'chunks':256,'building_interiors':54,'cars':len(set(m['group'] for m in full['faceMaterials'] if m['group'].startswith('Cars/'))),'train_wagons':25,'source':'JBRS 0.7.5, LOD0 world + high-detail car meshes; fresh initial fleet snapshot','notes':['Quad faces retain the source triangle diagonal 0-2. No world geometry was culled.','Box primitives have separate vertex indices so the editor can select them independently. The optional primitives table identifies their vertices/faces; the editor uses the standard vertices/faces/faceMaterials fields.','Runtime physics, scripts, lighting and animations are not part of this editor format. Cars and the freight train are static snapshots.','Colors are exported; shader material IDs and UV atlas effects are not supported.']}
(a.output/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n');print(manifest)
