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
    fn ribbon(&self, i: usize, width: f32) -> [Vec3; 4] {
        let a = self.points[i];
        let b = self.points[i + 1];
        let (nx, nz) = self.normal(i);
        let (bx, bz) = self.normal(i + 1);
        let edge = |p: Vec3, dx: f32, dz: f32| {
            let cross = if self.name != "Mountain tunnel" && terrain::height(p.x, p.z) >= -0.2 {
                deck(p.x + dx, p.z + dz) - deck(p.x, p.z)
            } else {
                0.
            };
            Vec3::new(p.x + dx, p.y + cross, p.z + dz)
        };
        [
            edge(a, nx * width, nz * width),
            edge(b, bx * width, bz * width),
            edge(b, -bx * width, -bz * width),
            edge(a, -nx * width, -nz * width),
        ]
    }
    /// Height on the actual ribbon triangles, rather than a lower neighboring segment.
    pub fn covered_height(&self, p: Vec3) -> Option<f32> {
        let mut height: Option<f32> = None;
        for i in 0..self.points.len().saturating_sub(1) {
            let a = self.points[i];
            let b = self.points[i + 1];
            let margin = self.width + 4.;
            if p.x < a.x.min(b.x) - margin
                || p.x > a.x.max(b.x) + margin
                || p.z < a.z.min(b.z) - margin
                || p.z > a.z.max(b.z) + margin
            {
                continue;
            }
            let q = self.ribbon(i, self.width / 2. + 2.);
            for indices in [[0, 1, 2], [0, 2, 3]] {
                let [a, b, c] = indices.map(|j| q[j]);
                let denominator = (b.z - c.z) * (a.x - c.x) + (c.x - b.x) * (a.z - c.z);
                if denominator.abs() < 1e-6 {
                    continue;
                }
                let u = ((b.z - c.z) * (p.x - c.x) + (c.x - b.x) * (p.z - c.z)) / denominator;
                let v = ((c.z - a.z) * (p.x - c.x) + (a.x - c.x) * (p.z - c.z)) / denominator;
                let t = 1. - u - v;
                if u >= -0.0001 && v >= -0.0001 && t >= -0.0001 {
                    let y = a.y * u + b.y * v + c.y * t;
                    height = Some(height.map_or(y, |h| h.min(y)));
                }
            }
        }
        height
    }
    pub fn ground_cut(&self, p: Vec3) -> (f32, f32) {
        let (d, y) = self.distance(p);
        (d, self.covered_height(p).unwrap_or(y))
    }
    pub fn mesh(&self, key: (i32, i32), lod: u8, m: &mut Mesh) {
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
            let quad = |width: f32, dy: f32| self.ribbon(i, width).map(|p| [p.x, p.y + dy, p.z]);
            m.quad(quad(self.width / 2. + 2., 0.025), [0.52, 0.54, 0.52], 1, 0.);
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
    let base = if (x - terrain::river_center(z)).abs() < 60. && (-500. ..700.).contains(&z) {
        let t = 1. - ((x - terrain::river_center(z)).abs() / 60.).clamp(0., 1.);
        2. * t * t * (3. - 2. * t)
    } else if x > 420. && (-260. ..500.).contains(&z) {
        terrain::industrial_height(x, z)
    } else if (280. ..740.).contains(&z) && (x - (-610. + 24. * (z * 0.012).sin())).abs() < 42. {
        2. * (1. - ((x - (-610. + 24. * (z * 0.012).sin())).abs() / 42.).clamp(0., 1.))
    } else {
        0.
    };
    // The south perimeter crosses the western lake as well as the river.
    // A broad approach keeps the lake bridge above water without a step.
    let lake_radius = ((x + 540.) / 115.).hypot((z - 430.) / 90.);
    let t = ((1.45 - lake_radius) / 0.75).clamp(0., 1.);
    base.max(2. * t * t * (3. - 2. * t))
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
    // Keep the new shop's marked east entrance clear of the north/south street.
    for road in &mut r {
        if road.name == "City street" && (road.points[0].x - 160.).abs() < 0.1 {
            for p in &mut road.points {
                let t = (-p.z / 40.).clamp(0., 1.);
                p.x = 160. + (engine_core::locations::donut::ROAD_X - 160.) * t;
                p.y = deck(p.x, p.z);
            }
        }
    }
    r.push(spline(
        "Donut approach",
        &[
            (engine_core::locations::donut::ROAD_X, -80.),
            (216., -80.),
            (210., -80.),
        ],
        7.5,
        false,
    ));
    // Ordinary streets complete the west bank and southern perimeter from the map.
    // Keep these after the original roads so existing spawn anchors stay stable.
    for (name, control) in [
        (
            "West bank street",
            vec![(-560., -260.), (-560., 0.), (-560., 140.), (-560., 300.)],
        ),
        (
            "North bridge link",
            vec![
                (-680., -280.),
                (-560., -280.),
                (-350., -280.),
                (-280., -260.),
                (0., -300.),
            ],
        ),
        (
            "South perimeter",
            vec![
                (-700., 300.),
                (-680., 400.),
                (-520., 450.),
                (-300., 450.),
                (0., 450.),
                (160., 450.),
                (480., 450.),
                (620., 430.),
            ],
        ),
        (
            "South city street",
            vec![(-120., 300.), (-120., 380.), (-120., 450.)],
        ),
        (
            "East perimeter",
            vec![
                (740., 0.),
                (800., 60.),
                (820., 120.),
                (800., 240.),
                (740., 300.),
            ],
        ),
    ] {
        let mut control = control;
        // Snap new endpoints to existing samples; visual proximity alone does not
        // guarantee a junction in the traffic graph.
        for index in [0, control.len() - 1] {
            let (x, z) = control[index];
            if let Some(p) = r
                .iter()
                .flat_map(|road| &road.points)
                .filter(|p| (p.x - x).hypot(p.z - z) < 20.)
                .min_by(|a, b| {
                    (a.x - x)
                        .hypot(a.z - z)
                        .total_cmp(&(b.x - x).hypot(b.z - z))
                })
            {
                control[index] = (p.x, p.z);
            }
        }
        r.push(spline(name, &control, 7.5, false));
    }
    align_rail_crossings(&mut r);
    stitch_junctions(&mut r);
    r
}

/// Shared endpoints use one height, including airport ramps entering flat access roads.
pub fn stitch_junctions(roads: &mut [Road]) {
    let ends: Vec<_> = roads
        .iter()
        .flat_map(|r| [r.points[0], *r.points.last().unwrap()])
        .collect();
    for road in roads {
        for end in [0, road.points.len() - 1] {
            let p = road.points[end];
            let target = ends
                .iter()
                .filter(|q| q.distance(p) < 1.)
                .map(|q| q.y)
                .fold(p.y, f32::max);
            if target - p.y < 0.01 {
                continue;
            }
            for i in 0..road.points.len() {
                let distance = road.points[i].distance(p);
                if distance < 70. {
                    road.points[i].y = road.points[i].y.max(target - distance * 0.035);
                }
            }
        }
    }
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
    fn extension_routes_join_city_and_cross_water_above_the_bed() {
        let roads = network();
        let graph = routing::Network::new(&roads);
        for road in roads.iter().filter(|r| {
            matches!(
                r.name,
                "West bank street"
                    | "North bridge link"
                    | "South perimeter"
                    | "South city street"
                    | "East perimeter"
            )
        }) {
            for p in [road.points[0], *road.points.last().unwrap()] {
                assert!(
                    graph.route(p, Vec3::new(0., 0., 0.)).is_some(),
                    "{} disconnected at {p:?}",
                    road.name
                );
            }
            for p in &road.points {
                if terrain::height(p.x, p.z) < -0.5 {
                    assert!(p.y > 0.5, "{} submerged at {p:?}", road.name);
                }
            }
        }
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
