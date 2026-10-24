"""Compile the supplied editor model into immutable geometry and component collisions."""
import json,struct,math,re
from pathlib import Path
root=Path(__file__).resolve().parent
scale=float(re.search(r'pub const SCALE:\s*f32\s*=\s*([0-9.]+)',(root.parents[3]/'engine_core/src/locations.rs').read_text()).group(1))
d=json.loads((root/'model.json').read_text());v=d['vertices'];f=d['faces'];m=d['faceMaterials'];assert len(f)==len(m)
assert all(len(p)==3 and all(math.isfinite(x) for x in p) for p in v)
parent=list(range(len(v)))
def find(i):
 while parent[i]!=i:parent[i]=parent[parent[i]];i=parent[i]
 return i
for face in f:
 assert len(face)>=3 and all(isinstance(i,int) and 0<=i<len(v) for i in face)
 for i in face:parent[find(i)]=find(face[0])
groups={}
for i in range(len(v)):groups.setdefault(find(i),[]).append(i)
components={find(ids[0]):number for number,ids in enumerate(groups.values())}
# Door panels, glass inserts and handles are independent components in this supplied model.
left={78,79,82};right={80,81,83};bins={k:bytearray() for k in ['model','roof','left','right']};collisions=[];glass=[]
def point(p,part):
 x,y,z=p
 if part in ['left','right']:x-=(-2.05 if part=='left' else 2.05);z-=-7.95
 return [-z*scale,y*scale,x*scale]
for component,ids in enumerate(groups.values()):
 faces=[i for i,face in enumerate(f) if components[find(face[0])]==component]
 lo=[min(v[i][k] for i in ids) for k in range(3)];hi=[max(v[i][k] for i in ids) for k in range(3)]
 part='left' if component in left else ('right' if component in right else ('roof' if any(m[i].get('group')=='roof' for i in faces) else 'model'))
 transparent=all(m[i].get('opacity',1)<.99 for i in faces)
 points=[point(v[i],part) for i in ids];a=[min(p[k] for p in points) for k in range(3)];b=[max(p[k] for p in points) for k in range(3)];center=[(x+y)/2 for x,y in zip(a,b)];size=[max(.01,y-x) for x,y in zip(a,b)]
 if part not in ['left','right'] and hi[1]>.12:
  collisions.append((center,size))
 if transparent:
  color=m[faces[0]].get('color','#b4e978');rgb=[int(color[k:k+2],16)/255 for k in [1,3,5]]
  glass.append((center,size,rgb,{'model':0,'roof':0,'left':-1,'right':1}[part]));continue
 for fi in faces:
  face=f[fi];color=m[fi].get('color','#b4e978');rgb=[int(color[k:k+2],16)/255 for k in [1,3,5]]
  for j in range(1,len(face)-1):
   for i in [face[0],face[j],face[j+1]]:bins[part].extend(struct.pack('<7f',*point(v[i],part),*rgb,0.))
for key,data in bins.items():(root/(key+'.bin')).write_bytes(data)
def arr(x):return '['+','.join(f'{y:.7f}' for y in x)+']'
text='// Generated from assets/model.json by assets/compile.py.\n'
text+='pub const COLLISIONS: &[([f32;3],[f32;3])] = &[\n'+''.join(f'({arr(p)},{arr(s)}),\n' for p,s in collisions)+'];\n'
text+='pub type Glass=([f32;3],[f32;3],[f32;3],i8);\npub const GLASS: &[Glass] = &[\n'+''.join(f'({arr(p)},{arr(s)},{arr(c)},{part}),\n' for p,s,c,part in glass)+'];\n'
(root/'generated.rs').write_text(text);print({k:len(x)//84 for k,x in bins.items()},'colliders',len(collisions),'glass panes',len(glass))
