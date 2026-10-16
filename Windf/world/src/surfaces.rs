//! One terrain sampler shared by rendered ground and contact physics.
use super::*;
pub(crate) fn ground_sample<'a>(
    x: f32,
    z: f32,
    roads: impl Iterator<Item = &'a Road>,
    buildings: impl Iterator<Item = &'a Building>,
) -> f32 {
    let raw = terrain::height(x, z);
    if (-380. ..500.).contains(&x) && (-350. ..350.).contains(&z) && raw == 0. {
        return 0.;
    }
    let mut ground = raw;
    for b in buildings {
        if b.name == "Penthouse" {
            continue;
        }
        let d = ((x - b.p.x).abs() - b.size.x / 2.).max((z - b.p.z).abs() - b.size.z / 2.);
        if d < 7. {
            let t = ((d - 3.) / 4.).clamp(0., 1.);
            ground = b.p.y * (1. - t) + raw * t;
            break;
        }
    }
    let p = Vec3::new(x, 0., z);
    if let Some((d, y, width, tunnel)) = roads
        .map(|r| {
            let (d, y) = r.ground_cut(p);
            (d, y, r.width, r.name == "Mountain tunnel")
        })
        .min_by(|a, b| {
            let ac = a.0 < a.2 / 2. + 10.;
            let bc = b.0 < b.2 / 2. + 10.;
            match (ac, bc) {
                (true, true) => a.1.total_cmp(&b.1),
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.0.total_cmp(&b.0),
            }
        })
    {
        // Keep the river bed below elevated bridges and mountain cover over the tunnel.
        if !(tunnel && raw > y + 6. || raw < -0.2 && y > 0.5) {
            let t = ((d - width / 2. - 10.) / 10.).clamp(0., 1.);
            ground = y * (1. - t) + ground * t;
        }
    }
    ground
}
pub(crate) fn road_tiles(roads: &[Road]) -> HashMap<(i32, i32), Vec<Road>> {
    let mut tiles = HashMap::new();
    for x in -9..=8 {
        for z in -9..=8 {
            let mut out = Vec::new();
            let (x0, z0) = (x as f32 * CHUNK, z as f32 * CHUNK);
            for r in roads {
                let mut run = Vec::new();
                let flush = |run: &mut Vec<Vec3>, out: &mut Vec<Road>| {
                    if run.len() > 1 {
                        out.push(Road {
                            name: r.name,
                            points: std::mem::take(run),
                            width: r.width,
                            highway: r.highway,
                        });
                    } else {
                        run.clear();
                    }
                };
                for seg in r.points.windows(2) {
                    let (a, b) = (seg[0], seg[1]);
                    if a.x.min(b.x) < x0 + CHUNK + 40.
                        && a.x.max(b.x) > x0 - 40.
                        && a.z.min(b.z) < z0 + CHUNK + 40.
                        && a.z.max(b.z) > z0 - 40.
                    {
                        if run.is_empty() {
                            run.push(a);
                        }
                        run.push(b);
                    } else {
                        flush(&mut run, &mut out);
                    }
                }
                flush(&mut run, &mut out);
            }
            if !out.is_empty() {
                tiles.insert((x, z), out);
            }
        }
    }
    tiles
}
impl World {
    pub(crate) fn local_roads(&self, x: f32, z: f32) -> &[Road] {
        self.surface_roads
            .get(&((x / CHUNK).floor() as i32, (z / CHUNK).floor() as i32))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    pub fn ground_height(&self, x: f32, z: f32) -> f32 {
        ground_sample(x, z, self.local_roads(x, z).iter(), self.buildings.iter())
    }
    pub fn vehicle_surface(&self, p: Vec3) -> f32 {
        self.surface(p.x, p.z)
    }
    pub fn water_depth(&self, p: Vec3) -> f32 {
        (-0.2 - self.ground_height(p.x, p.z)).max(0.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn traffic_lanes_and_train_envelope_are_clear() {
        let w = World::new();
        for p in terrain::railway::samples() {
            assert!(
                !w.blocked(Vec3::new(p.x, p.y + 2., p.z), 2.),
                "rail obstruction {p:?}"
            );
            for b in &w.buildings {
                assert!(
                    (p.x - b.p.x).abs() > b.size.x / 2. + 3.
                        || (p.z - b.p.z).abs() > b.size.z / 2. + 3.,
                    "track through {}",
                    b.name
                );
            }
        }
        for r in &w.roads {
            for (i, p) in r.points.iter().enumerate() {
                let (nx, nz) = r.normal(i);
                for side in [-1., 1.] {
                    let q = Vec3::new(p.x + nx * side * 1.6, p.y + 0.65, p.z + nz * side * 1.6);
                    assert!(!w.blocked(q, 1.), "lane blocked {} {q:?}", r.name);
                    if r.name != "Mountain tunnel" {
                        assert!(
                            w.ground_height(q.x, q.z) <= p.y + 0.15,
                            "ground covers road {} {q:?}",
                            r.name
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn forest_is_deterministic_irregular_and_avoids_transport() {
        let a = World::new();
        let b = World::new();
        assert!(a.trees.len() > 200);
        assert_eq!(a.trees.len(), b.trees.len());
        for (p, q) in a.trees.iter().zip(b.trees.iter()) {
            assert_eq!(p.x, q.x);
            assert_eq!(p.z, q.z);
            assert!(terrain::railway::nearest(*p).0 >= 9.);
            assert!(a
                .roads
                .iter()
                .all(|r| r.distance(*p).0 >= r.width / 2. + 8.));
        }
        let off_grid = a
            .trees
            .iter()
            .filter(|p| (p.x / 46.).fract().abs() > 0.05 && (p.z / 43.).fract().abs() > 0.05)
            .count();
        assert!(off_grid > a.trees.len() / 2);
    }
    #[test]
    fn water_mesh_is_flat_above_a_real_riverbed() {
        let z = 420.;
        let x = terrain::river_center(z);
        let w = World::new();
        assert!(w.water_depth(Vec3::new(x, 0., z)) > 1.);
        let key = ((x / 128.).floor() as i32, (z / 128.).floor() as i32);
        let m = w.chunk(key, 0);
        let water: Vec<_> = m
            .vertices
            .iter()
            .filter(|v| v.uv[0] < 0.25 && v.uv[1] >= 0.5 && v.uv[1] < 0.75)
            .collect();
        assert!(!water.is_empty());
        // Water material 8 occupies its own atlas tile; it is a surface, not a collider.
        for v in water {
            assert!((v.position[1] + 0.2).abs() < 0.001);
        }
        assert!(!w.blocked(Vec3::new(x, -0.1, z), 0.5));
    }
}
