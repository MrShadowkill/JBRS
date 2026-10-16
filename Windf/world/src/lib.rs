mod city_blocks;
mod geography;
pub mod prison;
mod streaming;
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
        let mut world = Self {
            roads,
            buildings: library.buildings,
            colliders,
            props,
            pois,
            trees,
            cells: HashMap::new(),
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
        if prison::underground(p) {
            return self.blocked(p, r);
        }
        if p.x.abs() > 1010. || p.z.abs() > 1010. {
            return true;
        }
        self.cells.get(&cell(p.x, p.z)).is_some_and(|ids| {
            ids.iter().any(|i| {
                let c = &self.colliders[*i];
                p.y - 1. + height > c.p.y - c.size.y / 2.
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
            if from.distance(entry) > 4. {
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
                let clear = !self.blocked(p, 0.45) && (p.y - previous_height).abs() < 0.6;
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
            if expanded > 100 {
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
                if (q.0 - origin.0).abs() > 24 || (q.1 - origin.1).abs() > 24 {
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
        while let Some(prev) = parent.get(&q) {
            if *prev == origin {
                break;
            }
            q = *prev;
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
        if let Some(id) = self.interior(p) {
            return self.buildings[id].support(p);
        }
        // The central plain and its roads are at sea-level; avoid scanning every
        // spline segment during local navigation on this known flat region.
        if (-380. ..500.).contains(&x) && (-350. ..350.).contains(&z) && terrain::height(x, z) == 0.
        {
            return 0.;
        }
        if let Some(b) = self.buildings.iter().find(|b| {
            b.name != "Penthouse"
                && (x - b.p.x).abs() < b.size.x / 2. + 7.
                && (z - b.p.z).abs() < b.size.z / 2. + 7.
        }) {
            let d = ((x - b.p.x).abs() - b.size.x / 2.).max((z - b.p.z).abs() - b.size.z / 2.);
            let blend = ((d - 3.) / 4.).clamp(0., 1.);
            return b.p.y * (1. - blend) + terrain::height(x, z) * blend;
        }
        let mut nearest = (f32::MAX, 0., 0.);
        for road in &self.roads {
            let (d, y) = road.distance(p);
            if d < nearest.0 {
                nearest = (d, y, road.width);
            }
        }
        if nearest.0 < nearest.2 / 2. + 1.5 {
            return nearest.1;
        }
        if nearest.0 < nearest.2 / 2. + 10. && terrain::height(x, z) < 20. {
            let blend = ((nearest.0 - nearest.2 / 2. - 5.) / 5.).clamp(0., 1.);
            nearest.1 * (1. - blend) + terrain::height(x, z) * blend
        } else {
            terrain::height(x, z)
        }
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
        let mut m = terrain::mesh_reusing(key, lod, vertices);
        let x0 = key.0 as f32 * CHUNK;
        let z0 = key.1 as f32 * CHUNK;
        let in_chunk = |p: Vec3| p.x >= x0 && p.x < x0 + CHUNK && p.z >= z0 && p.z < z0 + CHUNK;
        // Smooth road embankments; preserve mountain cover above the tunnel.
        let nearby: Vec<&Road> = self
            .roads
            .iter()
            .filter(|r| {
                r.points
                    .iter()
                    .any(|p| p.x > x0 - 32. && p.x < x0 + 160. && p.z > z0 - 32. && p.z < z0 + 160.)
            })
            .collect();
        for v in &mut m.vertices {
            let p = Vec3::new(v.position[0], 0., v.position[2]);
            if v.position[1] == -0.2 {
                continue;
            }
            for r in &nearby {
                let (d, y) = r.distance(p);
                if d < r.width / 2. + 10. && v.position[1] < 20. {
                    let blend = ((d - r.width / 2. - 5.) / 5.).clamp(0., 1.);
                    v.position[1] = y * (1. - blend) + v.position[1] * blend;
                }
            }
        }
        self.railway_mesh(key, &mut m);
        if key == (-1, -1) {
            prison::mesh(&mut m, lod);
        }
        for v in &mut m.vertices {
            if v.position[1] < -0.25 || v.position[1] > 25. {
                continue;
            }
            for b in &self.buildings {
                if b.name == "Penthouse" {
                    continue;
                }
                let dx = (v.position[0] - b.p.x).abs() - b.size.x / 2.;
                let dz = (v.position[2] - b.p.z).abs() - b.size.z / 2.;
                let d = dx.max(dz);
                if d < 7. {
                    let blend = ((d - 3.) / 4.).clamp(0., 1.);
                    v.position[1] = b.p.y * (1. - blend) + v.position[1] * blend;
                }
            }
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
        // Rail corridor shares the transport grid but is physically independent.
        for i in 0..16 {
            let x = x0 + i as f32 * 8.;
            let z = 365.;
            if in_chunk(Vec3::new(x, 0., z)) {
                m.cuboid([x, 0.08, z], [1., 0.16, 3.5], [0.47, 0.32, 0.2], 10, 0.);
                for zz in [z - 1., z + 1.] {
                    m.cuboid([x, 0.2, zz], [8.1, 0.15, 0.12], [0.62, 0.64, 0.66], 4, 0.);
                }
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
        for i in 0..600 {
            let a = i as f32 / 600. * std::f32::consts::TAU;
            let b = (i + 1) as f32 / 600. * std::f32::consts::TAU;
            let x = 800. * a.cos();
            let z = 750. * a.sin();
            if cell(x, z) != key {
                continue;
            }
            let qx = 800. * b.cos();
            let qz = 750. * b.sin();
            let y = terrain::height(x, z).max(0.) + 0.2;
            let qy = terrain::height(qx, qz).max(0.) + 0.2;
            let dx = qx - x;
            let dz = qz - z;
            let len = (dx * dx + dz * dz).sqrt();
            let nx = -dz / len;
            let nz = dx / len;
            for side in [-1., 1.] {
                let off = side * 1.1;
                m.quad(
                    [
                        [x + nx * (off - 0.06), y, z + nz * (off - 0.06)],
                        [qx + nx * (off - 0.06), qy, qz + nz * (off - 0.06)],
                        [qx + nx * (off + 0.06), qy, qz + nz * (off + 0.06)],
                        [x + nx * (off + 0.06), y, z + nz * (off + 0.06)],
                    ],
                    [0.55, 0.56, 0.58],
                    4,
                    0.,
                );
            }
            m.cuboid([x, y - 0.15, z], [3., 0.25, 0.4], [0.35, 0.22, 0.12], 2, 0.);
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
