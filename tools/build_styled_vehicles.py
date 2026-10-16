"""Build original low-poly vehicles matching the supplied model's visual language.
Python standard library only. Run from any directory; files resolve relative to this script.
All generated geometry is original. The imported Akari is handled by import_akari.py.
"""
from pathlib import Path
import math
import struct

ROOT = Path(__file__).resolve().parents[1]
DARK = (0.055, 0.06, 0.07)
RUBBER = (0.045, 0.047, 0.052)
METAL = (0.67, 0.69, 0.72)
GLASS = (0.27, 0.33, 0.43)
WHITE = (1., 1., 1.)
# length, half width, roof height, wheelbase, cabin rear/base/roof/front,
# front deck/rear deck. Units are metres, origin is 0.65m above the ground.
PROFILES = [
    ('vesper', 4.5, 1.0, .87, 2.72, (-1.35,-.85,.35,1.02), .36,.42),
    ('mica', 3.65,.94,1.04,2.35,(-1.6,-1.38,.22,.92),.31,.48),
    ('sentinel',4.5,1.05,1.02,2.82,(-1.5,-1.,.45,1.17),.4,.46),
    ('voltura',4.5,1.04,.79,2.87,(-1.85,-.83,.48,1.36),.25,.37),
    ('ridgeback',5.1,1.06,1.22,3.12,(-.37,-.2,.85,1.35),.56,.54),
    ('kestrel',4.8,1.08,.72,2.82,(-1.65,-.72,.55,1.25),.24,.36),
    ('aether',4.55,1.12,.68,2.78,(-1.5,-.58,.35,1.15),.21,.33),
]

def sub(a,b): return tuple(x-y for x,y in zip(a,b))
def cross(a,b): return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def unit(v):
    d=math.sqrt(sum(x*x for x in v))
    return tuple(x/d for x in v) if d>1e-10 else (0.,1.,0.)

class Mesh:
    def __init__(self): self.vertices=[]
    def tri(self,a,b,c,color,paint=False,normal=None):
        n=unit(normal or cross(sub(b,a),sub(c,a)))
        # Face orientation is explicit where needed; use height-lit bevels and restrained contrast.
        light=.66+.34*max(0.,sum(x*y for x,y in zip(n,unit((-.35,.8,.45)))))
        for p in (a,b,c): self.vertices.append((*p,*(x*light for x in color),float(paint)))
    def quad(self,p,color,paint=False,normal=None):
        self.tri(p[0],p[1],p[2],color,paint,normal)
        self.tri(p[0],p[2],p[3],color,paint,normal)
    def box(self,p,s,c,paint=False):
        v=[(p[0]+x*s[0]/2,p[1]+y*s[1]/2,p[2]+z*s[2]/2) for z in (-1,1) for y in (-1,1) for x in (-1,1)]
        for ids,n in [((0,1,3,2),(0,0,-1)),((4,6,7,5),(0,0,1)),((0,2,6,4),(-1,0,0)),((1,5,7,3),(1,0,0)),((2,3,7,6),(0,1,0)),((0,4,5,1),(0,-1,0))]: self.quad([v[i] for i in ids],c,paint,n)
    def beam(self,a,b,r,c,paint=False):
        d=unit(sub(b,a));u=unit(cross(d,(1,0,0) if abs(d[1])>.9 else (0,1,0)));v=cross(d,u)
        ring=lambda p:[tuple(p[j]+r*(s*u[j]+t*v[j]) for j in range(3)) for s,t in [(-1,-1),(-1,1),(1,1),(1,-1)]]
        aa,bb=ring(a),ring(b)
        for i in range(4): self.quad([aa[i],aa[(i+1)%4],bb[(i+1)%4],bb[i]],c,paint)
    def save(self,path):
        data=b'JBV1'+struct.pack('<I',len(self.vertices)//3)+b''.join(struct.pack('<7f',*v) for v in self.vertices)
        path.write_bytes(data)
        print(path.name,len(self.vertices)//3,'triangles')

def wheel(m,x,z,r,detail,style):
    n=20 if detail else 8
    side=1 if x>0 else -1
    at=lambda d,rr,a:(x+side*d,-.3+rr*math.cos(a),z+rr*math.sin(a))
    rings=[(-.14,r*.78),(-.1,r),(.1,r),(.15,r*.84),(.153,r*.67)]
    for i in range(n):
        a,b=i*math.tau/n,(i+1)*math.tau/n
        for (d0,r0),(d1,r1) in zip(rings,rings[1:]): m.quad([at(d0,r0,a),at(d1,r1,a),at(d1,r1,b),at(d0,r0,b)],RUBBER,normal=(side, .2, 0))
        m.tri(at(.155,0,a),at(.155,r*.67,a),at(.155,r*.67,b),(.19,.2,.22),normal=(side,0,0))
        m.quad([at(.16,r*.66,a),at(.16,r*.74,a),at(.16,r*.74,b),at(.16,r*.66,b)],METAL,normal=(side,0,0))
    if detail:
        spokes=5 if style!=2 else 8
        for i in range(spokes):
            a=i*math.tau/spokes
            m.quad([at(.17,r*.16,a-.35),at(.17,r*.65,a-.12),at(.17,r*.65,a+.12),at(.17,r*.16,a+.35)],METAL,normal=(side,0,0))
        m.box((x+side*.165,-.3,z),(.018,.1,.1),(.42,.45,.5))
        m.box((x+side*.157,-.3,z+r*.45),(.016,.14,.08),(.65,.09,.055))

def build(kind,detail):
    name,length,w,roof,base,cabin,hood,rear=PROFILES[kind]
    m=Mesh();half=length/2;r=.35 if kind!=4 else .39
    # Shoulder and underside are separate surfaces. Side skirts rise around each wheel,
    # leaving genuine openings rather than painting arcs onto a closed box.
    zs={-half,half,-half+.18,half-.18,0.}
    for i in range(1,17 if detail else 7): zs.add(-half+length*i/(17 if detail else 7))
    for zc in (-base/2,base/2):
        for i in range(17 if detail else 7):
            a=math.pi*i/(16 if detail else 6)
            zs.add(zc+(r+.065)*math.cos(a))
    zs=sorted(z for z in zs if -half<=z<=half)
    def surface(z):
        t=(z+half)/length
        width=w*(.87+.13*math.sin(math.pi*t)**.4)
        if kind>=5: width*=1.-.04*math.exp(-(z/.6)**2)
        deck=rear+(hood-rear)*t
        deck-=.12*(abs(z)/half)**5
        bottom=-.34
        for zc in (-base/2,base/2):
            d=abs(z-zc)
            if d<r+.065: bottom=max(bottom,-.3+math.sqrt((r+.065)**2-d*d))
        return width,deck,bottom
    for za,zb in zip(zs,zs[1:]):
        wa,ya,ba=surface(za);wb,yb,bb=surface(zb)
        for side in (-1,1):
            # Multiple rounded shoulder strips replace the old single sharp bevel.
            bands=5 if detail else 2
            for band in range(bands):
                a=band*math.pi/2/bands;b=(band+1)*math.pi/2/bands
                point=lambda ww,yy,zz,t:(side*ww*(.78+.22*math.sin(t)),yy+.025-.15*(1.-math.cos(t)),zz)
                m.quad([point(wa,ya,za,a),point(wb,yb,zb,a),point(wb,yb,zb,b),point(wa,ya,za,b)],WHITE,True,(side*math.sin((a+b)/2),math.cos((a+b)/2),0))
            m.quad([(side*wa,ya-.125,za),(side*wb,yb-.125,zb),(side*wb,bb,zb),(side*wa,ba,za)],WHITE,True,(side,.15,0))
            m.quad([(side*wa,ba,za),(side*wb,bb,zb),(side*wb*.83,bb-.035,zb),(side*wa*.83,ba-.035,za)],(.72,.72,.72),True,(side,-.2,0))
        for band in range(6 if detail else 2):
            n=6 if detail else 2;aa=-1.+2.*band/n;bbx=-1.+2.*(band+1)/n
            point=lambda ww,yy,zz,t:(t*ww*.78,yy+.025+.025*(1.-t*t),zz)
            m.quad([point(wa,ya,za,aa),point(wb,yb,zb,aa),point(wb,yb,zb,bbx),point(wa,ya,za,bbx)],WHITE,True,((aa+bbx)*.08,1,0))
    for z in (-half,half):
        ww,yy,bb=surface(z)
        m.quad([(-ww,bb,z),(ww,bb,z),(ww,yy-.065,z),(-ww,yy-.065,z)],WHITE,True,(0,.1,1 if z>0 else -1))
    rb,rr,rf,fb=cabin
    waist=(rear+hood)/2+.015
    rw=w*(.69 if kind<5 else .65)
    # Rounded roof strips give the same restrained faceting as the reference model.
    for i in range(6 if detail else 2):
        n=6 if detail else 2
        x0=-rw+2*rw*i/n;x1=-rw+2*rw*(i+1)/n
        yy=lambda x:roof-.045*(x/rw)**2
        m.quad([(x0,yy(x0),rr),(x1,yy(x1),rr),(x1,yy(x1),rf),(x0,yy(x0),rf)],WHITE,True,(0,1,0))
    for side in (-1,1):
        # Framed side glazing follows the cabin rake. Two windows for four-door bodies.
        points=[(side*w*.78,waist,rb),(side*rw,roof-.045,rr),(side*rw,roof-.045,rf),(side*w*.78,waist,fb)]
        m.quad(points,DARK,normal=(side,.1,0))
        center=tuple(sum(p[j] for p in points)/4 for j in range(3))
        inset=[tuple(center[j]+.91*(p[j]-center[j])+(side*.006 if j==0 else 0) for j in range(3)) for p in points]
        m.quad(inset,GLASS,normal=(side,.25,0))
        if detail:
            blend=lambda a,b,t:tuple(x+(y-x)*t for x,y in zip(a,b))
            m.quad([blend(inset[0],inset[1],.7),inset[1],blend(inset[2],inset[3],.2),blend(inset[2],inset[3],.3)],(.38,.44,.54),normal=(side,.2,0))
        for a,b in zip(points,points[1:]+points[:1]):m.beam(a,b,.022,WHITE,True)
        if detail:
            mid=(rr+rf)/2
            m.beam((side*w*.785,waist,mid),(side*rw,roof-.045,mid),.025,DARK)
            for dz in [rb+.14,fb-.13]:m.beam((side*w*1.005,-.1,dz),(side*w*1.005,waist-.05,dz),.006,DARK)
            m.box((side*w*1.007,waist-.075,mid-.23),(.02,.035,.17),(.24,.26,.29))
            if kind in (1,2,3): m.beam((side*w*1.005,-.1,mid),(side*w*1.005,waist-.025,mid),.006,DARK)
        m.box((side*(w+.075),waist+.1,fb-.18),(.21,.115,.2),WHITE,True)
        m.box((side*(w+.075),waist+.1,fb-.285),(.16,.075,.012),GLASS)
        m.box((side*w*.99,-.29,0.),(.07,.08,base-.7),(.65,.65,.65),True)
    for end,base_z,top_z in [(-1,rb,rr),(1,fb,rf)]:
        p=[(-w*.77,waist+.012,base_z),(-rw,roof-.047,top_z),(rw,roof-.047,top_z),(w*.77,waist+.012,base_z)]
        m.quad(p,GLASS,normal=(0,.65,end*.7))
        for a,b in zip(p,p[1:]+p[:1]):m.beam(a,b,.018,WHITE,True)
        # Baked reflection panel on the glass, no transparency sorting or modern shaders.
        a,b,c,d=p
        mix=lambda x,y,t:tuple(v+(q-v)*t for v,q in zip(x,y))
        m.quad([mix(a,b,.12),mix(a,b,.2),mix(d,c,.2),mix(d,c,.12)],(.4,.46,.56),normal=(0,1,0))
    for side in (-1,1):
        for z in (-base/2,base/2):wheel(m,side*(w-.08),z,r,detail,kind)
        fw,fy,_=surface(half)
        if kind==0 or kind==5:
            for offset in (.5,.73):
                x=side*offset
                for i in range(12 if detail else 6):
                    n=12 if detail else 6;a=i*math.tau/n;b=(i+1)*math.tau/n
                    m.tri((x,fy-.06,half+.012),(x+.095*math.cos(a),fy-.06+.095*math.sin(a),half+.012),(x+.095*math.cos(b),fy-.06+.095*math.sin(b),half+.012),(.94,.92,.76),normal=(0,0,1))
        else:
            m.box((side*fw*.63,fy-.06,half+.014),(.47,.085 if kind>=3 else .13,.025),(.9,.93,.96))
        m.box((side*w*.61,rear-.14,-half-.012),(.48,.095,.035),(.68,.045,.035))
        m.box((side*.62,-.24,-half-.07),(.17,.095,.12),METAL)
        m.box((side*.62,-.24,-half-.135),(.115,.06,.012),DARK)
    m.box((0.,hood-.15,half+.02),(w*1.1,.1,.035),DARK)
    m.box((0.,-.25,half+.025),(w*1.7,.075,.08),DARK)
    m.box((0.,-.25,-half-.025),(w*1.7,.075,.08),DARK)
    for z in [-half-.045,half+.045]:m.box((0.,.02,z),(.32,.095,.012),(.77,.78,.74))
    if detail:
        for side in (-1,1):
            m.beam((side*.6,hood+.01,half-.2),(side*.52,waist+.02,fb),.007,(.83,.83,.83),True)
            m.beam((side*.5,waist+.03,fb-.02),(side*.24,waist+.085,fb-.15),.009,DARK)
        for y in [hood-.17,hood-.13]:m.box((0.,y,half+.04),(w*.9,.012,.012),METAL)
    if kind==1:
        m.box((0.,roof-.12,rb-.025),(1.25,.04,.035),(.6,.04,.03))
    if kind==2:
        m.box((0.,roof+.06,-.1),(1.45,.07,.28),DARK)
        for side in (-1,1):
            m.box((side*.43,roof+.15,-.1),(.46,.13,.24),(.8,.03,.035) if side<0 else (.03,.15,.75))
            m.box((side*w*1.008,.12,0.),(.012,.2,.62),(.82,.83,.85))
        if detail:m.beam((.6,roof,-.6),(.6,roof+.5,-.6),.008,DARK)
    if kind==3:m.box((0.,rear-.09,-half-.035),(1.6,.025,.012),(.72,.025,.025))
    if kind==4:
        # The bed occupies only the rear deck; cab and tailgate remain distinct.
        m.box((0.,rear+.02,-1.42),(1.76,.035,1.65),(.17,.18,.19))
        for side in (-1,1):
            m.box((side*.96,rear+.2,-1.42),(.14,.4,1.85),WHITE,True)
            m.box((side*.96,rear+.41,-1.42),(.15,.025,1.85),DARK)
            m.box((side*(w+.1),-.32,.4),(.22,.07,1.55),METAL)
        m.box((0.,rear+.2,-2.35),(1.85,.4,.08),WHITE,True)
        if detail:
            for x in range(-5,6):m.box((x*.14,rear+.045,-1.42),(.025,.02,1.55),(.25,.26,.27))
            m.box((0.,rear+.31,-2.395),(.28,.04,.015),DARK)
    if kind>=5:
        for side in (-1,1):
            m.box((side*w*1.002,.06,-.52),(.035,.21,.6),DARK)
            if detail:
                for i in range(4):m.box((side*w*1.023,.0+i*.045,-.52),(.015,.012,.5),(.29,.31,.34))
        m.box((0.,-.32,half),(w*1.95,.045,.22),DARK)
        m.box((0.,-.31,-half),(w*1.8,.055,.25),DARK)
        if detail:
            for x in [-.6,-.3,0.,.3,.6]:m.box((x,-.25,-half),(.02,.17,.25),DARK)
    if kind==5:
        m.box((0.,.72,-half+.23),(2.1,.065,.32),DARK)
        for side in (-1,1):m.box((side*.64,.52,-half+.23),(.04,.37,.12),METAL)
    if kind==6:
        # Flying buttresses and engine cover distinguish this from the front-engine GT.
        for side in (-1,1):
            m.beam((side*.66,roof-.03,rr),(side*.88,rear,-half+.25),.055,WHITE,True)
        if detail:
            for i in range(7):m.box((0.,rear+.035,-half+.3+i*.1),(.9,.018,.035),DARK)
    return m

if __name__=='__main__':
    target=ROOT/'vehicles/assets';target.mkdir(parents=True,exist_ok=True)
    for kind,p in enumerate(PROFILES):
        for lod,detail in [('high',True),('low',False)]:build(kind,detail).save(target/f'{p[0]}-{lod}.jbv')
