mod city_blocks;
mod geography;
pub mod prison;
mod streaming;
mod surfaces;
use buildings::{Building, Collider, Library};
use engine_core::{Mesh, Vec3};
use roads::Road;
use std::collections::HashMap;
pub use streaming::{ChunkData, Streamer};
pub const EXTENT: f32 = 1024.;
pub const CHUNK: f32 = 128.;
#[derive(Clone)]
pub struct Poi {
    pub name: &'static str,
    pub p: Vec3,
    pub color: [f32; 3],
}
#[derive(Clone)]
pub struct Prop {
    pub p: Vec3,
    pub size: Vec3,
    pub material: u8,
    pub color: [f32; 3],
    pub emission: f32,
    pub roof: bool,
}
pub struct World {
    pub roads: Vec<Road>,
    pub buildings: Vec<Building>,
    pub colliders: Vec<Collider>,
    pub props: Vec<Prop>,
    pub pois: Vec<Poi>,
    pub trees: Vec<Vec3>,
    cells: HashMap<(i32, i32), Vec<usize>>,
    surface_roads: HashMap<(i32, i32), Vec<Road>>,
}
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
impl World {
    pub fn new() -> Self {
        geography::build()
    }
    pub(crate) fn assemble(
        roads: Vec<Road>,
        library: Library,
        colliders: Vec<Collider>,
        props: Vec<Prop>,
        pois: Vec<Poi>,
        trees: Vec<Vec3>,
    ) -> Self {
        let surface_roads = surfaces::road_tiles(&roads);
        let mut world = Self {
            roads,
            buildings: library.buildings,
            colliders,
            props,
            pois,
            trees,
            cells: HashMap::new(),
            surface_roads,
        };
        for b in &world.buildings {
            world.colliders.extend(b.colliders());
        }
        for t in &world.trees {
            let h = world.surface(t.x, t.z);
            world.colliders.push(Collider {
                p: Vec3::new(t.x, h + 2., t.z),
                size: Vec3::new(0.6, 4., 0.6),
                movable: false,
                breakable: true,
            });
        }
        for p in terrain::railway::samples().iter().step_by(4) {
            if world
                .roads
                .iter()
                .any(|r| r.distance(*p).0 < r.width / 2. + 8.)
                || world.buildings.iter().any(|b| {
                    (p.x - b.p.x).abs() < b.size.x / 2. + 6.
                        && (p.z - b.p.z).abs() < b.size.z / 2. + 6.
                })
            {
                continue;
            }
            let ground = world.surface(p.x, p.z);
            let height = p.y - 0.32 - ground;
            if height > 1. {
                let center = Vec3::new(p.x, ground + height / 2., p.z);
                let size = Vec3::new(0.65, height, 0.65);
                world.props.push(Prop {
                    p: center,
                    size,
                    material: 1,
                    color: [0.43, 0.45, 0.44],
                    emission: 0.,
                    roof: false,
                });
                world.colliders.push(Collider {
                    p: center,
                    size,
                    movable: false,
                    breakable: false,
                });
            }
        }
        for (i, c) in world.colliders.iter().enumerate() {
            let min = cell(c.p.x - c.size.x / 2. - 4., c.p.z - c.size.z / 2. - 4.);
            let max = cell(c.p.x + c.size.x / 2. + 4., c.p.z + c.size.z / 2. + 4.);
            for x in min.0..=max.0 {
                for z in min.1..=max.1 {
                    world.cells.entry((x, z)).or_default().push(i);
                }
            }
        }
        world
    }
    pub fn blocked(&self, p: Vec3, r: f32) -> bool {
        if prison::underground(p) {
            return !prison::passable(p, r);
        }
        if p.x.abs() > 1010. || p.z.abs() > 1010. {
            return true;
        }
        if self
            .interior(p)
            .is_some_and(|i| self.buildings[i].support(p) > p.y - 0.5)
        {
            return true;
        }
        self.cells
            .get(&cell(p.x, p.z))
            .is_some_and(|ids| ids.iter().any(|i| self.colliders[*i].contains(p, r)))
    }
    pub fn body_blocked(&self, p: Vec3, r: f32, height: f32) -> bool {
        self.body_blocked_where(p, r, height, |_| true)
    }
    pub fn body_blocked_where(
        &self,
        p: Vec3,
        r: f32,
        height: f32,
        active: impl Fn(&Collider) -> bool,
    ) -> bool {
        if prison::underground(p) {
            return self.blocked(p, r);
        }
        if p.x.abs() > 1010. || p.z.abs() > 1010. {
            return true;
        }
        self.cells.get(&cell(p.x, p.z)).is_some_and(|ids| {
            ids.iter().any(|i| {
                let c = &self.colliders[*i];
                active(c)
                    && p.y - 1. + height > c.p.y - c.size.y / 2.
                    && p.y - 1. < c.p.y + c.size.y / 2.
                    && (p.x - c.p.x).abs() < c.size.x / 2. + r
                    && (p.z - c.p.z).abs() < c.size.z / 2. + r
            })
        })
    }
    pub fn walking_surface(&self, p: Vec3) -> f32 {
        if prison::underground(p) {
            -4.
        } else if let Some(id) = self.interior(p) {
            self.buildings[id].support(p)
        } else {
            self.surface(p.x, p.z)
        }
    }
    pub fn next_step(&self, from: Vec3, target: Vec3, distance: f32) -> Vec3 {
        let original_target = target;
        let start_interior = self.interior(from);
        // Route through an actual entrance rather than getting stuck at the nearest wall.
        let target = if let Some(id) = self.interior(from) {
            let b = &self.buildings[id];
            if !b.inside(target) {
                b.entrance()
            } else {
                target
            }
        } else if let Some(id) = self.interior(target) {
            let entry = self.buildings[id].entrance();
            if from.distance(entry) > 4. || (from.x - entry.x).abs() > 0.4 {
                entry
            } else {
                target
            }
        } else {
            target
        };
        let steer = |goal: Vec3| {
            let dx = goal.x - from.x;
            let dz = goal.z - from.z;
            let len = (dx * dx + dz * dz).sqrt().max(0.001);
            let step = distance.min(len);
            let mut p = Vec3::new(from.x + dx / len * step, from.y, from.z + dz / len * step);
            p.y = self.walking_surface(p) + 1.;
            p
        };
        let direct = steer(target);
        let clear = |a: Vec3, b: Vec3| {
            let samples = (a.distance(b) / 0.4).ceil().clamp(1., 2048.) as i32;
            let mut previous_height = self.walking_surface(a) + 1.;
            (1..=samples).all(|i| {
                let t = i as f32 / samples as f32;
                let mut p = Vec3::new(a.x + (b.x - a.x) * t, a.y, a.z + (b.z - a.z) * t);
                p.y = self.walking_surface(p) + 1.;
                let reenters = start_interior.is_none()
                    && self
                        .interior(p)
                        .is_some_and(|id| !self.buildings[id].inside(original_target));
                let clear =
                    !reenters && !self.blocked(p, 0.45) && (p.y - previous_height).abs() < 0.6;
                previous_height = p.y;
                clear
            })
        };
        if clear(from, target) {
            return direct;
        }
        // A 48m local search; node spacing matches the 2m cell door clearance.
        let origin = (from.x.round() as i32, from.z.round() as i32);
        let mut frontier = std::collections::BinaryHeap::new();
        let mut cost = HashMap::new();
        let mut parent = HashMap::new();
        frontier.push((0, origin));
        cost.insert(origin, 0i32);
        let heuristic =
            |q: (i32, i32)| ((q.0 as f32 - target.x).abs() + (q.1 as f32 - target.z).abs()) as i32;
        let mut best = origin;
        let mut expanded = 0;
        while let Some((_, node)) = frontier.pop() {
            expanded += 1;
            if expanded > 768 {
                break;
            }
            if heuristic(node) < heuristic(best) {
                best = node;
            }
            let a = Vec3::new(node.0 as f32, from.y, node.1 as f32);
            if heuristic(node) < 2 {
                best = node;
                break;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let q = (node.0 + dx, node.1 + dz);
                if (q.0 - origin.0).abs() > 32 || (q.1 - origin.1).abs() > 32 {
                    continue;
                }
                let b = Vec3::new(q.0 as f32, from.y, q.1 as f32);
                if !clear(a, b) {
                    continue;
                }
                let g = cost[&node] + 1;
                if cost.get(&q).is_none_or(|old| g < *old) {
                    cost.insert(q, g);
                    parent.insert(q, node);
                    frontier.push((-(g + heuristic(q)), q));
                }
            }
        }
        let mut q = best;
        let mut path = vec![q];
        while let Some(prev) = parent.get(&q) {
            if *prev == origin {
                break;
            }
            q = *prev;
            path.push(q);
        }
        // Reuse a visible stretch of the path for the next AI planning interval.
        for node in path.iter().rev() {
            let candidate = Vec3::new(node.0 as f32, from.y, node.1 as f32);
            if from.distance(candidate) > distance + 0.5 {
                break;
            }
            if clear(from, candidate) {
                q = *node;
            } else {
                break;
            }
        }
        let p = steer(Vec3::new(q.0 as f32, from.y, q.1 as f32));
        if clear(from, p) {
            p
        } else {
            // A fractional position beside a jamb may not reach the first grid
            // neighbor diagonally. Align with the clear origin before turning.
            let aligned = steer(Vec3::new(origin.0 as f32, from.y, origin.1 as f32));
            if from.distance(aligned) > 0.001 && clear(from, aligned) {
                aligned
            } else {
                from
            }
        }
    }
    pub fn ceiling(&self, p: Vec3) -> f32 {
        if prison::underground(p) {
            return -0.3;
        }
        if let Some(id) = self.interior(p) {
            let b = &self.buildings[id];
            if b.name == "Donut Shop" {
                return b.p.y + 5.98;
            }
            if b.name == "Jewelry" && p.y > b.p.y + 12.5 {
                return f32::MAX;
            }
            if b.name == "Jewelry" {
                if let Some(height) = buildings::jewelry::ramp_height(b, p) {
                    return height + 4.;
                }
                return b.p.y + ((p.y - b.p.y - 0.6) / 4.).floor().max(0.) * 4. + 4.;
            }
            if matches!(b.name, "Office" | "Westhaven") {
                return b.support(p) + 4.;
            }
            return b.p.y + b.size.y;
        }
        if p.z < -460. {
            if let Some(r) = self.roads.iter().find(|r| r.name == "Mountain tunnel") {
                if r.distance(p).0 < 6. {
                    return 6.;
                }
            }
        }
        f32::MAX
    }
    pub fn surface(&self, x: f32, z: f32) -> f32 {
        let p = Vec3::new(x, 0., z);
        if let Some((distance, y, width)) = self
            .local_roads(x, z)
            .iter()
            .map(|road| {
                let (d, y) = road.ground_cut(p);
                (d, y, road.width)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
        {
            if distance < width / 2. + 2. {
                return y;
            }
        }
        self.ground_height(x, z)
    }
    pub fn interior(&self, p: Vec3) -> Option<usize> {
        self.buildings.iter().position(|b| {
            b.interior
                && b.inside(p)
                && p.y + 1.5 >= b.p.y
                && p.y - 0.8 <= b.p.y + b.size.y + if b.name == "Jewelry" { 6.5 } else { 0. }
        })
    }
    pub fn chunk(&self, key: (i32, i32), lod: u8) -> Mesh {
        self.chunk_reusing(key, lod, Vec::new())
    }
    pub fn chunk_reusing(
        &self,
        key: (i32, i32),
        lod: u8,
        vertices: Vec<engine_core::Vertex>,
    ) -> Mesh {
        let x0 = key.0 as f32 * CHUNK;
        let z0 = key.1 as f32 * CHUNK;
        let in_chunk = |p: Vec3| p.x >= x0 && p.x < x0 + CHUNK && p.z >= z0 && p.z < z0 + CHUNK;
        let nearby = self.local_roads(x0, z0);
        let pads: Vec<&Building> = self
            .buildings
            .iter()
            .filter(|b| {
                b.p.x + b.size.x / 2. + 8. > x0
                    && b.p.x - b.size.x / 2. - 8. < x0 + CHUNK
                    && b.p.z + b.size.z / 2. + 8. > z0
                    && b.p.z - b.size.z / 2. - 8. < z0 + CHUNK
            })
            .collect();
        let mut m = terrain::mesh_sampled(key, lod, vertices, |x, z| {
            surfaces::ground_sample(x, z, nearby.iter(), pads.iter().copied())
        });
        self.railway_mesh(key, &mut m);
        if key == (-1, -1) {
            prison::mesh(&mut m, lod);
        }
        for road in &self.roads {
            road.mesh(key, lod, &mut m);
        }
        for b in &self.buildings {
            if in_chunk(b.p) {
                b.mesh(&mut m, lod);
            }
        }
        for prop in &self.props {
            if in_chunk(prop.p) && (lod == 0 || prop.size.y > 5.) {
                if prop.roof {
                    m.roof(
                        [prop.p.x, prop.p.y, prop.p.z],
                        [prop.size.x, prop.size.y, prop.size.z],
                        prop.color,
                    );
                } else {
                    m.cuboid(
                        [prop.p.x, prop.p.y, prop.p.z],
                        [prop.size.x, prop.size.y, prop.size.z],
                        prop.color,
                        prop.material,
                        prop.emission,
                    );
                }
            }
        }
        for t in &self.trees {
            if in_chunk(*t) {
                let h = self.surface(t.x, t.z);
                if lod == 0 {
                    m.cylinder([t.x, h, t.z], 0.3, 4., [0.55, 0.4, 0.25], 10, 6);
                    for (y, r) in [(3., 3.8), (5., 2.8), (7., 1.6)] {
                        for i in 0..6 {
                            let a = i as f32 * std::f32::consts::TAU / 6.;
                            let b = (i + 1) as f32 * std::f32::consts::TAU / 6.;
                            m.triangle(
                                [
                                    [t.x + a.cos() * r, h + y, t.z + a.sin() * r],
                                    [t.x + b.cos() * r, h + y, t.z + b.sin() * r],
                                    [t.x, h + y + 3.5, t.z],
                                ],
                                [0.21, 0.4, 0.25],
                                6,
                                0.,
                            );
                        }
                    }
                } else {
                    m.triangle(
                        [
                            [t.x - 3., h + 2., t.z],
                            [t.x + 3., h + 2., t.z],
                            [t.x, h + 10., t.z],
                        ],
                        [0.24, 0.42, 0.26],
                        6,
                        0.,
                    );
                    m.triangle(
                        [
                            [t.x, h + 2., t.z - 3.],
                            [t.x, h + 2., t.z + 3.],
                            [t.x, h + 10., t.z],
                        ],
                        [0.24, 0.42, 0.26],
                        6,
                        0.,
                    );
                }
            }
        }
        // Sparse crossed grass blades on open grassy ground; nearby chunks have
        // a denser tuft LOD while distant chunks retain only occasional silhouettes.
        let grass_step = if lod == 0 { 10.0 } else { 24.0 };
        let grass_stride = if lod == 0 { 2 } else { 4 };
        for ix in 0..(CHUNK / grass_step) as i32 {
            for iz in 0..(CHUNK / grass_step) as i32 {
                if (ix * 13 + iz * 7).rem_euclid(grass_stride) != 0 {
                    continue;
                }
                let x = x0 + (ix as f32 + 0.37) * grass_step;
                let z = z0 + (iz as f32 + 0.61) * grass_step;
                let raw = terrain::height(x, z);
                if raw < -0.2
                    || terrain::material(x, z, raw).0 != 6
                    || self
                        .roads
                        .iter()
                        .any(|road| road.distance(Vec3::new(x, raw, z)).0 < road.width * 0.5 + 2.0)
                    || self
                        .trees
                        .iter()
                        .any(|tree| tree.distance(Vec3::new(x, raw, z)) < 6.0)
                    || pads.iter().any(|b| {
                        (x - b.p.x).abs() < b.size.x * 0.5 + 3.0
                            && (z - b.p.z).abs() < b.size.z * 0.5 + 3.0
                    })
                    || self
                        .props
                        .iter()
                        .any(|prop| prop.p.distance(Vec3::new(x, raw, z)) < 3.0)
                    || terrain::railway::nearest(Vec3::new(x, raw, z)).0 < 10.0
                {
                    continue;
                }
                let y = self.surface(x, z) + 0.03;
                let height = if lod == 0 { 0.48 } else { 0.34 };
                let half = if lod == 0 { 0.085 } else { 0.11 };
                let green = [0.30, 0.48, 0.24];
                m.swaying_triangle(
                    [[x - half, y, z], [x + half, y, z], [x, y + height, z]],
                    green,
                    6,
                );
                m.swaying_triangle(
                    [[x, y, z - half], [x, y, z + half], [x, y + height, z]],
                    [0.36, 0.53, 0.27],
                    6,
                );
            }
        }
        // Mountain tunnel has continuous visible inner walls and a clear 6m envelope.
        if lod == 0 {
            for r in &self.roads {
                if r.name == "Mountain tunnel" {
                    for (i, seg) in r.points.windows(2).enumerate() {
                        let a = seg[0];
                        let b = seg[1];
                        if in_chunk(a) && a.z < -460. {
                            let (nx, nz) = r.normal(i);
                            let (bx, bz) = r.normal(i + 1);
                            for side in [-1., 1.] {
                                m.quad(
                                    [
                                        [a.x + nx * side * 6., 0., a.z + nz * side * 6.],
                                        [b.x + bx * side * 6., 0., b.z + bz * side * 6.],
                                        [b.x + bx * side * 6., 6., b.z + bz * side * 6.],
                                        [a.x + nx * side * 6., 6., a.z + nz * side * 6.],
                                    ],
                                    [0.58, 0.59, 0.55],
                                    1,
                                    0.,
                                );
                            }
                            m.quad(
                                [
                                    [a.x + nx * 6., 6., a.z + nz * 6.],
                                    [b.x + bx * 6., 6., b.z + bz * 6.],
                                    [b.x - bx * 6., 6., b.z - bz * 6.],
                                    [a.x - nx * 6., 6., a.z - nz * 6.],
                                ],
                                [0.45, 0.48, 0.47],
                                1,
                                0.,
                            );
                        }
                    }
                }
            }
        }
        m
    }
    pub fn navigation_image(&self) -> Vec<u8> {
        let n = 256;
        let mut pixels = vec![0u8; n * n * 3];
        for z in 0..n {
            for x in 0..n {
                let wx = x as f32 / 256. * 2048. - 1024.;
                let wz = z as f32 / 256. * 2048. - 1024.;
                let h = terrain::height(wx, wz);
                let (_, c) = terrain::material(wx, wz, h);
                let i = (z * n + x) * 3;
                for j in 0..3 {
                    pixels[i + j] = (c[j] * 170.) as u8;
                }
            }
        }
        let mut dot = |x: f32, z: f32, r: i32, c: [u8; 3]| {
            let px = ((x + 1024.) / 2048. * 256.) as i32;
            let pz = ((z + 1024.) / 2048. * 256.) as i32;
            for dz in -r..=r {
                for dx in -r..=r {
                    let xx = px + dx;
                    let zz = pz + dz;
                    if (0..256).contains(&xx) && (0..256).contains(&zz) {
                        let i = (zz as usize * 256 + xx as usize) * 3;
                        pixels[i..i + 3].copy_from_slice(&c);
                    }
                }
            }
        };
        for road in &self.roads {
            for seg in road.points.windows(2) {
                let a = seg[0];
                let b = seg[1];
                for i in 0..8 {
                    let t = i as f32 / 8.;
                    dot(
                        a.x + (b.x - a.x) * t,
                        a.z + (b.z - a.z) * t,
                        if road.highway { 1 } else { 0 },
                        [155, 161, 166],
                    );
                }
            }
        }
        for b in &self.buildings {
            for x in (-(b.size.x / 2.) as i32..=(b.size.x / 2.) as i32).step_by(8) {
                for z in (-(b.size.z / 2.) as i32..=(b.size.z / 2.) as i32).step_by(8) {
                    dot(b.p.x + x as f32, b.p.z + z as f32, 0, [91, 107, 121]);
                }
            }
        }
        pixels
    }
    pub fn audit(&self) -> Result<(), String> {
        for b in &self.buildings {
            let hx = b.size.x / 2.;
            let hz = b.size.z / 2.;
            for road in &self.roads {
                for p in &road.points {
                    if (p.x - b.p.x).abs() < hx + road.width / 2. - 0.2
                        && (p.z - b.p.z).abs() < hz + road.width / 2. - 0.2
                    {
                        return Err(format!(
                            "Road {} overlaps building {} #{} at {},{}",
                            road.name, b.name, b.id, p.x, p.z
                        ));
                    }
                }
            }
        }
        for b in &self.buildings {
            if b.interior {
                let mut p = b.entrance();
                for i in 0..60 {
                    p.z = b.entrance().z + i as f32 * 0.5;
                    if p.z > b.p.z {
                        break;
                    }
                    if self.blocked(p, 0.45) {
                        return Err(format!("Interior entrance blocked: {}", b.name));
                    }
                }
            }
        }
        Ok(())
    }
}
pub fn cell(x: f32, z: f32) -> (i32, i32) {
    ((x / CHUNK).floor() as i32, (z / CHUNK).floor() as i32)
}

impl World {
    fn railway_mesh(&self, key: (i32, i32), m: &mut Mesh) {
        for (i, segment) in terrain::railway::samples().windows(2).enumerate() {
            let a = segment[0];
            let b = segment[1];
            if cell((a.x + b.x) / 2., (a.z + b.z) / 2.) != key {
                continue;
            }
            let len = (b.x - a.x).hypot(b.z - a.z).max(0.001);
            let nx = -(b.z - a.z) / len;
            let nz = (b.x - a.x) / len;
            let middle = Vec3::new((a.x + b.x) * 0.5, 0.0, (a.z + b.z) * 0.5);
            let crossing = self
                .roads
                .iter()
                .any(|road| road.distance(middle).0 < road.width / 2.0 + 5.0);
            let ground_a = terrain::height(a.x, a.z);
            let ground_b = terrain::height(b.x, b.z);
            if !crossing && ground_a >= -0.35 && ground_b >= -0.35 {
                for side in [-1.0, 1.0] {
                    let top_y = a.y - 0.32;
                    let other_top_y = b.y - 0.32;
                    let base_a = ground_a.min(top_y - 0.05);
                    let base_b = ground_b.min(other_top_y - 0.05);
                    if top_y > base_a + 0.15 || other_top_y > base_b + 0.15 {
                        let inner = 2.7 * side;
                        let outer = 8.0 * side;
                        m.quad(
                            [
                                [a.x + nx * inner, top_y, a.z + nz * inner],
                                [b.x + nx * inner, other_top_y, b.z + nz * inner],
                                [b.x + nx * outer, base_b, b.z + nz * outer],
                                [a.x + nx * outer, base_a, a.z + nz * outer],
                            ],
                            [0.42, 0.35, 0.27],
                            1,
                            0.0,
                        );
                    }
                }
            }
            let strip = |offset: f32, width: f32, dy: f32| {
                [
                    [
                        a.x + nx * (offset - width),
                        a.y + dy,
                        a.z + nz * (offset - width),
                    ],
                    [
                        b.x + nx * (offset - width),
                        b.y + dy,
                        b.z + nz * (offset - width),
                    ],
                    [
                        b.x + nx * (offset + width),
                        b.y + dy,
                        b.z + nz * (offset + width),
                    ],
                    [
                        a.x + nx * (offset + width),
                        a.y + dy,
                        a.z + nz * (offset + width),
                    ],
                ]
            };
            m.quad(strip(0., 2.7, -0.32), [0.38, 0.38, 0.35], 1, 0.);
            for off in [-1.1, 1.1] {
                m.quad(strip(off, 0.065, 0.05), [0.62, 0.65, 0.67], 4, 0.);
            }
            for f in [0., 0.25, 0.5, 0.75] {
                let x = a.x + (b.x - a.x) * f;
                let y = a.y + (b.y - a.y) * f - 0.08;
                let z = a.z + (b.z - a.z) * f;
                m.quad(
                    [
                        [x + nx * 2.35 - nz * 0.16, y, z + nz * 2.35 + nx * 0.16],
                        [x + nx * 2.35 + nz * 0.16, y, z + nz * 2.35 - nx * 0.16],
                        [x - nx * 2.35 + nz * 0.16, y, z - nz * 2.35 - nx * 0.16],
                        [x - nx * 2.35 - nz * 0.16, y, z - nz * 2.35 + nx * 0.16],
                    ],
                    [0.34, 0.25, 0.16],
                    2,
                    0.,
                );
            }
            let _ = i;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn elevated_factory_mesh_collision_and_surface_use_same_base() {
        let w = World::new();
        let b = w
            .buildings
            .iter()
            .find(|b| b.name == "Factory" && b.p.x == 640.)
            .unwrap();
        assert!(b.p.y > 15.);
        assert_eq!(w.surface(b.p.x, b.p.z), b.p.y);
        assert!(b
            .colliders()
            .iter()
            .all(|c| c.p.y - c.size.y / 2. >= b.p.y - 0.01));
        let mut mesh = Mesh::default();
        b.mesh(&mut mesh, 0);
        assert!(mesh.vertices.iter().all(|v| v.position[1] >= b.p.y - 0.41));
        assert!(w.blocked(Vec3::new(b.p.x + b.size.x / 2., b.p.y + 1., b.p.z), 0.45));
        assert!(!w.blocked(Vec3::new(b.p.x, b.p.y + 1., b.p.z), 0.45));
    }
    #[test]
    fn every_cell_has_clear_doorway() {
        let w = World::new();
        for z in [-200., -190., -180., -170.] {
            for side in [-1., 1.] {
                for step in 0..=14 {
                    let p = Vec3::new(-145. + side * step as f32 * 0.5, 1., z);
                    assert!(!w.blocked(p, 0.45), "cell door {},{}", p.x, p.z);
                }
            }
        }
    }
    #[test]
    fn navigation_exits_cell_block_without_crossing_walls() {
        let w = World::new();
        let mut p = Vec3::new(-145., 1., -183.);
        let goal = Vec3::new(-100., 1., -160.);
        for _ in 0..400 {
            p = w.next_step(p, goal, 0.5);
            assert!(!w.blocked(p, 0.45));
            if p.distance(goal) < 1. {
                break;
            }
        }
        assert!(p.distance(goal) < 1., "stuck at {},{}", p.x, p.z);
    }
    #[test]
    fn map_audit() {
        World::new().audit().unwrap();
    }
    #[test]
    fn prison_perimeter_has_no_free_gate_exit() {
        let w = World::new();
        for x in -169..-31 {
            assert!(w.blocked(Vec3::new(x as f32, 1., -100.), 0.45));
        }
        for z in -219..-101 {
            let p = Vec3::new(-170., 1., z as f32);
            assert!(w.blocked(p, 0.45) || prison::barrier(p, 0.45, false));
        }
        assert!(prison::barrier(prison::WEAK, 0.45, false));
        assert!(!prison::barrier(prison::WEAK, 0.45, true));
    }
    #[test]
    fn deterministic_rebuild() {
        let a = World::new();
        let b = World::new();
        assert_eq!(a.colliders.len(), b.colliders.len());
        let ma = a.chunk((2, 0), 0);
        let mb = b.chunk((2, 0), 0);
        assert_eq!(ma.vertices.len(), mb.vertices.len());
        assert_eq!(ma.vertices[55].position, mb.vertices[55].position);
    }
    #[test]
    fn grass_chunks_have_sway_tags_and_lod_reduces_blades() {
        let w = World::new();
        let key = (-6, -2);
        let close = w.chunk(key, 0);
        let close_count = close.vertices.iter().filter(|v| v.color[3] < 0.).count();
        let distant = w.chunk(key, 1);
        let distant_count = distant.vertices.iter().filter(|v| v.color[3] < 0.).count();
        assert!(close_count > 0, "open grass chunk has no tagged blades");
        assert!(
            close_count > distant_count,
            "grass LOD did not reduce blade density"
        );
    }
    #[test]
    fn road_clearance() {
        let w = World::new();
        for road in &w.roads {
            for p in &road.points {
                assert!(
                    !w.blocked(Vec3::new(p.x, p.y + 0.65, p.z), 1.7),
                    "{} {},{}",
                    road.name,
                    p.x,
                    p.z
                );
            }
        }
    }
    #[test]
    fn bridge_and_tunnel_surface() {
        let w = World::new();
        assert!((w.surface(-440., 0.) - 2.).abs() < 0.1);
        for z in (-730..-500).step_by(5) {
            let road = w
                .roads
                .iter()
                .find(|r| r.name == "Mountain tunnel")
                .unwrap();
            let p = road
                .points
                .iter()
                .min_by(|a, b| (a.z - z as f32).abs().total_cmp(&(b.z - z as f32).abs()))
                .unwrap();
            assert_eq!(w.surface(p.x, p.z), p.y);
            assert!(!w.blocked(Vec3::new(p.x, 1., p.z), 1.7));
        }
    }
}
