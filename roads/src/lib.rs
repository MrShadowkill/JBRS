pub mod routing;
use engine_core::{Mesh, Vec3};
#[derive(Clone)]
pub struct Road {
    pub name: &'static str,
    pub points: Vec<Vec3>,
    pub width: f32,
    pub highway: bool,
}
impl Road {
    pub fn normal(&self, i: usize) -> (f32, f32) {
        let a = self.points[i.saturating_sub(1)];
        let b = self.points[(i + 1).min(self.points.len() - 1)];
        let len = (b.x - a.x).hypot(b.z - a.z).max(0.001);
        (-(b.z - a.z) / len, (b.x - a.x) / len)
    }
    pub fn distance(&self, p: Vec3) -> (f32, f32) {
        let mut best = (f32::MAX, 0.);
        for seg in self.points.windows(2) {
            let a = seg[0];
            let b = seg[1];
            let dx = b.x - a.x;
            let dz = b.z - a.z;
            let t = ((p.x - a.x) * dx + (p.z - a.z) * dz) / (dx * dx + dz * dz).max(0.001);
            let t = t.clamp(0., 1.);
            let q = Vec3::new(a.x + dx * t, a.y + (b.y - a.y) * t, a.z + dz * t);
            let d = ((p.x - q.x).powi(2) + (p.z - q.z).powi(2)).sqrt();
            if d < best.0 {
                best = (d, q.y);
            }
        }
        best
    }
    /// Lowest deck inside the terrain cutting envelope, including sharp joints.
    pub fn ground_cut(&self, p: Vec3) -> (f32, f32) {
        let (distance, mut height) = self.distance(p);
        if distance > self.width / 2. + 10. {
            return (distance, height);
        }
        for s in self.points.windows(2) {
            let a = s[0];
            let b = s[1];
            let (dx, dz) = (b.x - a.x, b.z - a.z);
            let t = (((p.x - a.x) * dx + (p.z - a.z) * dz) / (dx * dx + dz * dz).max(0.001))
                .clamp(0., 1.);
            let d = (p.x - a.x - dx * t).hypot(p.z - a.z - dz * t);
            if d < self.width / 2. + 3. {
                height = height.min(a.y + (b.y - a.y) * t);
            }
        }
        (distance, height)
    }
    pub fn mesh(&self, key: (i32, i32), lod: u8, m: &mut Mesh) {
        self.mesh_pass(key, lod, m, true, true);
    }

    fn mesh_pass(&self, key: (i32, i32), lod: u8, m: &mut Mesh, shoulder: bool, roadway: bool) {
        let x0 = key.0 as f32 * 128.;
        let z0 = key.1 as f32 * 128.;
        for (i, s) in self.points.windows(2).enumerate() {
            let a = s[0];
            let b = s[1];
            let middle = Vec3::new((a.x + b.x) / 2., 0., (a.z + b.z) / 2.);
            if middle.x < x0 - 20.
                || middle.x > x0 + 148.
                || middle.z < z0 - 20.
                || middle.z > z0 + 148.
            {
                continue;
            }
            let (nx, nz) = self.normal(i);
            let (bx, bz) = self.normal(i + 1);
            let quad = |width: f32, dy: f32| {
                [
                    [a.x + nx * width, a.y + dy, a.z + nz * width],
                    [b.x + bx * width, b.y + dy, b.z + bz * width],
                    [b.x - bx * width, b.y + dy, b.z - bz * width],
                    [a.x - nx * width, a.y + dy, a.z - nz * width],
                ]
            };
            if shoulder {
                m.quad(quad(self.width / 2. + 2., 0.025), [0.52, 0.54, 0.52], 1, 0.);
            }
            if !roadway {
                continue;
            }
            m.quad(quad(self.width / 2., 0.05), [0.32, 0.34, 0.37], 0, 0.);
            if i % 2 == 0 {
                m.quad(quad(0.1, 0.07), [0.97, 0.81, 0.31], 1, 0.);
            }
            if self.highway {
                for offset in [-3.5, 3.5] {
                    let shift = offset;
                    let mut q = quad(0.075, 0.08);
                    for v in &mut q {
                        v[0] += nx * shift;
                        v[2] += nz * shift;
                    }
                    m.quad(q, [0.9, 0.92, 0.88], 1, 0.);
                }
            }
            if lod == 0 && i % 6 == 0 {
                let x = a.x + nx * (self.width / 2. + 3.);
                let z = a.z + nz * (self.width / 2. + 3.);
                m.cuboid(
                    [x, a.y + 3.5, z],
                    [0.18, 7., 0.18],
                    [0.3, 0.33, 0.35],
                    4,
                    0.,
                );
                m.cuboid(
                    [x - nx, a.y + 7., z - nz],
                    [2.2, 0.2, 0.4],
                    [0.65, 0.66, 0.6],
                    4,
                    0.,
                );
                m.cuboid(
                    [x - nx, a.y + 6.85, z - nz],
                    [1.7, 0.06, 0.3],
                    [1., 0.85, 0.45],
                    9,
                    1.,
                );
            }
            if a.y > 0.8 && terrain::height(a.x, a.z) < 0. {
                for side in [-1., 1.] {
                    let mut q = quad(0.15, 1.);
                    for v in &mut q {
                        v[0] += nx * side * (self.width / 2. + 0.5);
                        v[2] += nz * side * (self.width / 2. + 0.5);
                    }
                    m.quad(q, [0.55, 0.57, 0.6], 4, 0.);
                }
                if i % 4 == 0 {
                    m.cuboid(
                        [a.x, a.y / 2. - 2., a.z],
                        [2., a.y + 4., 2.],
                        [0.5, 0.52, 0.5],
                        1,
                        0.,
                    );
                }
            }
        }
    }
}

/// Lay every road shoulder before any asphalt. At flat intersections, a later
/// road's pale shoulder must not paint over the dark deck of an earlier road.
pub fn mesh_network(roads: &[Road], key: (i32, i32), lod: u8, m: &mut Mesh) {
    for road in roads {
        road.mesh_pass(key, lod, m, true, false);
    }
    for road in roads {
        road.mesh_pass(key, lod, m, false, true);
    }
}
fn spline(name: &'static str, control: &[(f32, f32)], width: f32, highway: bool) -> Road {
    let mut points = Vec::new();
    for i in 0..control.len() - 1 {
        let p0 = control[i.saturating_sub(1)];
        let p1 = control[i];
        let p2 = control[i + 1];
        let p3 = control[(i + 2).min(control.len() - 1)];
        let count = (((p1.0 - p2.0).hypot(p1.1 - p2.1) / 8.).ceil() as usize).max(2);
        for j in 0..count {
            let t = j as f32 / count as f32;
            let f = |a: f32, b: f32, c: f32, d: f32| {
                0.5 * ((2. * b)
                    + (-a + c) * t
                    + (2. * a - 5. * b + 4. * c - d) * t * t
                    + (-a + 3. * b - 3. * c + d) * t * t * t)
            };
            let x = f(p0.0, p1.0, p2.0, p3.0);
            let z = f(p0.1, p1.1, p2.1, p3.1);
            points.push(Vec3::new(x, deck(x, z), z));
        }
    }
    let (x, z) = *control.last().unwrap();
    points.push(Vec3::new(x, deck(x, z), z));
    Road {
        name,
        points,
        width,
        highway,
    }
}
pub fn deck(x: f32, z: f32) -> f32 {
    if (x - terrain::river_center(z)).abs() < 60. && (-500. ..700.).contains(&z) {
        let t = 1. - ((x - terrain::river_center(z)).abs() / 60.).clamp(0., 1.);
        2. * t * t * (3. - 2. * t)
    } else if x > 420. && (-260. ..500.).contains(&z) {
        terrain::industrial_height(x, z)
    } else if (280. ..740.).contains(&z) && (x - (-610. + 24. * (z * 0.012).sin())).abs() < 42. {
        2. * (1. - ((x - (-610. + 24. * (z * 0.012).sin())).abs() / 42.).clamp(0., 1.))
    } else {
        0.
    }
}
pub fn network() -> Vec<Road> {
    let mut r = Vec::new();
    for z in [0., 140., 300.] {
        r.push(spline(
            "City avenue",
            &[(-740., z), (0., z), (480., z), (740., z)],
            14.,
            true,
        ));
    }
    for x in [0., 160., 480.] {
        r.push(spline(
            "City street",
            &[(x, if x == 0. { -140. } else { -300. }), (x, 450.)],
            7.5,
            false,
        ));
    }
    r.push(spline(
        "Bank approach",
        &[(320., -100.), (320., 0.), (320., 10.)],
        7.5,
        false,
    ));
    r.push(spline(
        "Prison connector",
        &[(-95., -76.), (-95., -40.), (-90., 0.)],
        7.5,
        false,
    ));
    r.push(spline(
        "Station boulevard",
        &[(-25., -140.), (20., -140.), (160., -140.), (480., -140.)],
        7.5,
        false,
    ));
    r.push(spline(
        "West shore road",
        &[(-720., -500.), (-680., -200.), (-680., 140.), (-700., 480.)],
        7.5,
        false,
    ));
    r.push(spline(
        "Airport express",
        &[(480., 300.), (620., 430.), (730., 650.)],
        14.,
        true,
    ));
    r.push(spline(
        "Terminal access",
        &[(730., 650.), (730., 685.), (700., 694.)],
        7.5,
        false,
    ));
    r.push(spline(
        "Harbor link",
        &[(480., 0.), (700., -40.), (820., 120.)],
        7.5,
        false,
    ));
    r.push(spline(
        "Desert route",
        &[(480., -140.), (680., -250.), (760., -450.), (620., -480.)],
        7.5,
        false,
    ));
    r.push(spline(
        "Mountain tunnel",
        &[
            (160., -300.),
            (180., -440.),
            (200., -580.),
            (220., -740.),
            (350., -820.),
            (530., -780.),
        ],
        9.,
        false,
    ));
    r.push(spline(
        "North ridge route",
        &[
            (-680., -200.),
            (-550., -300.),
            (-300., -360.),
            (0., -300.),
            (160., -300.),
        ],
        7.5,
        false,
    ));
    r.push(spline(
        "Criminal lane",
        &[(-250., 140.), (-250., 100.)],
        7.5,
        false,
    ));
    r.push(spline(
        "Prison service",
        &[(-170., -80.), (-175., -40.), (-95., -40.)],
        7.5,
        false,
    ));
    align_rail_crossings(&mut r);
    r
}

fn align_rail_crossings(roads: &mut [Road]) {
    const MAX_GRADE: f32 = 0.035;
    for road in roads {
        if road.points.len() < 3 {
            continue;
        }
        let distances: Vec<_> = road
            .points
            .iter()
            .map(|p| terrain::railway::nearest(*p).0)
            .collect();
        let mut crossings = Vec::new();
        for i in 1..road.points.len() - 1 {
            if distances[i] < 3.0
                && distances[i] <= distances[i - 1]
                && distances[i] <= distances[i + 1]
                && crossings.last().is_none_or(|last| i - *last > 2)
            {
                crossings.push(i);
            }
        }
        for center in crossings {
            let p = road.points[center];
            if terrain::height(p.x, p.z) < -0.35 {
                continue;
            }
            let rail_y = terrain::railway::nearest(p).1.y;
            let crossing_y = rail_y - 0.08;
            if p.y > crossing_y + 2.5 {
                // A clearly elevated deck remains a deliberate overpass.
                continue;
            }
            road.points[center].y = crossing_y;
            for direction in [-1isize, 1] {
                let mut distance = 0.0;
                let mut i = center as isize;
                loop {
                    let next = i + direction;
                    if next < 0 || next >= road.points.len() as isize {
                        break;
                    }
                    let previous = road.points[i as usize];
                    let next_point = road.points[next as usize];
                    distance += previous.distance(next_point);
                    i = next;
                    let original = next_point.y;
                    let allowance = MAX_GRADE * distance;
                    if (original - crossing_y).abs() <= allowance {
                        break;
                    }
                    road.points[i as usize].y = if original < crossing_y {
                        crossing_y - allowance
                    } else {
                        crossing_y + allowance
                    };
                }
            }
        }
    }
}

pub fn connected(roads: &[Road], start: Vec3, end: Vec3) -> bool {
    let nodes: Vec<Vec3> = roads
        .iter()
        .flat_map(|r| r.points.iter().copied())
        .collect();
    let mut seen = vec![false; nodes.len()];
    let Some(first) = nodes.iter().position(|n| n.distance(start) < 15.) else {
        return false;
    };
    seen[first] = true;
    let mut q = std::collections::VecDeque::from([first]);
    while let Some(i) = q.pop_front() {
        if nodes[i].distance(end) < 15. {
            return true;
        }
        for j in 0..nodes.len() {
            if !seen[j] && nodes[i].distance(nodes[j]) < 13. {
                seen[j] = true;
                q.push_back(j);
            }
        }
    }
    false
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn road_intersection_meshes_all_shoulders_below_all_road_decks() {
        let roads = [
            Road {
                name: "east west",
                points: vec![Vec3::new(-20., 0., 0.), Vec3::new(20., 0., 0.)],
                width: 8.,
                highway: false,
            },
            Road {
                name: "north south",
                points: vec![Vec3::new(0., 0., -20.), Vec3::new(0., 0., 20.)],
                width: 8.,
                highway: false,
            },
        ];
        let mut mesh = Mesh::default();
        mesh_network(&roads, (0, 0), 1, &mut mesh);
        assert_eq!(mesh.vertices.len(), 36);
        assert!(mesh.vertices[..12]
            .iter()
            .all(|v| v.color[..3] == [0.52, 0.54, 0.52]));
        assert!(mesh.vertices[12..18]
            .iter()
            .all(|v| v.color[..3] == [0.32, 0.34, 0.37]));
        assert!(mesh.vertices[18..24]
            .iter()
            .all(|v| v.color[..3] == [0.97, 0.81, 0.31]));
        assert!(mesh.vertices[24..30]
            .iter()
            .all(|v| v.color[..3] == [0.32, 0.34, 0.37]));
    }
    #[test]
    fn bridge_grade() {
        let r = network();
        for road in &r {
            for s in road.points.windows(2) {
                assert!(
                    (s[1].y - s[0].y).abs() / s[0].distance(s[1]) < 0.15,
                    "{} {:?} {:?}",
                    road.name,
                    s[0],
                    s[1]
                );
            }
        }
    }
    #[test]
    fn west_shore_rail_crossing_has_a_continuous_approach() {
        let roads = network();
        let road = roads
            .iter()
            .find(|road| road.name == "West shore road")
            .unwrap();
        let (index, point) = road
            .points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                terrain::railway::nearest(**a)
                    .0
                    .total_cmp(&terrain::railway::nearest(**b).0)
            })
            .unwrap();
        let (distance, rail) = terrain::railway::nearest(*point);
        assert!(distance < 3.0, "crossing miss: {distance} at {point:?}");
        assert!((point.y - (rail.y - 0.08)).abs() < 0.02);
        for direction in [-1isize, 1] {
            let mut i = index as isize;
            let mut walked = 0.0;
            while walked < 90.0 {
                let next = i + direction;
                if next < 0 || next >= road.points.len() as isize {
                    break;
                }
                let a = road.points[i as usize];
                let b = road.points[next as usize];
                let length = a.distance(b);
                assert!((b.y - a.y).abs() / length.max(0.001) <= 0.04);
                walked += length;
                i = next;
            }
        }
    }
    #[test]
    fn connected_destinations() {
        let r = network();
        for end in [
            Vec3::new(320., 0., 10.),
            Vec3::new(-250., 0., 100.),
            Vec3::new(730., 0., 650.),
            Vec3::new(820., 0., 120.),
        ] {
            assert!(
                connected(&r, Vec3::new(-95., 0., -76.), end),
                "{} {}",
                end.x,
                end.z
            );
        }
    }
}
