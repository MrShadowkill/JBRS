"""Run: blender -b -t 4 --python tools/import_akari.py -- /absolute/JBRS
Convert the supplied CC-BY GLB to embedded position/color triangle streams.
Blender is an authoring tool only; the game has no GLB or Blender dependency.
"""
import bpy
import json
import math
from pathlib import Path
import struct
import sys
from mathutils import Vector

root = Path(sys.argv[sys.argv.index('--') + 1]).resolve()
source = root / 'vehicles/assets/source/akari-r7.glb'
raw = source.read_bytes()
size = struct.unpack_from('<I', raw, 12)[0]
gltf = json.loads(raw[20:20 + size])
colors = {m['name']: m['emissiveFactor'] for m in gltf['materials']}
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(source))
objects = [o for o in bpy.context.scene.objects if o.type == 'MESH' and 'Floor' not in o.name]
coords = [o.matrix_world @ v.co for o in objects for v in o.data.vertices]
lo = Vector(tuple(min(v[i] for v in coords) for i in range(3)))
hi = Vector(tuple(max(v[i] for v in coords) for i in range(3)))
scale = 4.5 / (hi.x - lo.x)
center = (hi + lo) / 2
for o in objects:
    for v in o.data.vertices:
        p = o.matrix_world @ v.co
        # Blender Z-up to engine Y-up. Source nose is -Blender X; engine forward is +Z.
        v.co = (-(p.y - center.y) * scale, (p.z - lo.z) * scale - 0.65, -(p.x - center.x) * scale)
    o.matrix_world.identity()
    o.data.update()

# The source reuses Material_9 for both body skirts and wheel rims. Split its
# disconnected components before decimation so recoloring does not tint metal.
for obj in objects:
    if obj.data.materials[0].name != 'Material_9':
        continue
    mesh = obj.data
    parents = list(range(len(mesh.vertices)))
    def find(i):
        while parents[i] != i:
            parents[i] = parents[parents[i]]
            i = parents[i]
        return i
    def join(a, b):
        parents[find(a)] = find(b)
    positions = {}
    for vertex in mesh.vertices:
        key = tuple(round(v, 4) for v in vertex.co)
        if key in positions:
            join(vertex.index, positions[key])
        positions[key] = vertex.index
    for polygon in mesh.polygons:
        for vertex in polygon.vertices:
            join(polygon.vertices[0], vertex)
    components = {}
    for vertex in mesh.vertices:
        components.setdefault(find(vertex.index), []).append(vertex.co)
    painted = set()
    for component, points in components.items():
        low = [min(p[i] for p in points) for i in range(3)]
        high = [max(p[i] for p in points) for i in range(3)]
        center_point = [(a+b)/2 for a,b in zip(low,high)]
        dimensions = [b-a for a,b in zip(low,high)]
        x,y,z = center_point
        wheel = abs(x) > .65 and -.65 < y < 0 and .8 < abs(z) < 1.8 and dimensions[2] < .65
        rear_hardware = z < -1.9 and y < -.15 and dimensions[0] < .65 and dimensions[1] < .25
        if not wheel and not rear_hardware:
            painted.add(component)
    mesh.materials.append(mesh.materials[0])
    for polygon in mesh.polygons:
        polygon.material_index = int(find(polygon.vertices[0]) in painted)

for lod, ratio in [('high', 1.0), ('low', 0.18)]:
    output = bytearray(b'JBV1' + bytes(4))
    triangles = 0
    for obj in objects:
        clone = obj.copy()
        clone.data = obj.data.copy()
        bpy.context.collection.objects.link(clone)
        if ratio < 1 and len(clone.data.polygons) > 30:
            mod = clone.modifiers.new('Distance LOD', 'DECIMATE')
            mod.ratio = ratio
            mod.use_collapse_triangulate = True
        evaluated = clone.evaluated_get(bpy.context.evaluated_depsgraph_get())
        mesh = evaluated.to_mesh()
        mesh.calc_loop_triangles()
        material = obj.data.materials[0].name
        color = colors[material]
        for tri in mesh.loop_triangles:
            paint = material == '02_-_Default' or (material == 'Material_9' and mesh.polygons[tri.polygon_index].material_index == 1)
            triangles += 1
            for loop_id in tri.loops:
                loop = mesh.loops[loop_id]
                vertex = mesh.vertices[loop.vertex_index]
                normal = mesh.corner_normals[loop_id].vector if mesh.polygons[tri.polygon_index].use_smooth else tri.normal
                light = 0.64 + 0.36 * max(0, normal.dot(Vector((-0.35, 0.8, 0.45)).normalized()))
                # Authoring-time directional shading preserves the reference's smooth surfaces.
                rgb = [c * light for c in color]
                output.extend(struct.pack('<7f', *vertex.co, *rgb, float(paint)))
        evaluated.to_mesh_clear()
        bpy.data.objects.remove(clone, do_unlink=True)
    struct.pack_into('<I', output, 4, triangles)
    target = root / f'vehicles/assets/akari-{lod}.jbv'
    target.write_bytes(output)
    print(f'EXPORTED {target.name}: {triangles} triangles, {len(output)} bytes')
