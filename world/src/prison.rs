use engine_core::{Mesh, Vec3};
pub const HATCH: Vec3 = Vec3 {
    x: -116.,
    y: 1.,
    z: -145.,
};
pub const EXIT: Vec3 = Vec3 {
    x: -116.,
    y: 1.,
    z: -78.,
};
pub const ROUTE: [(f32, f32); 8] = [
    (-116., -145.),
    (-116., -132.),
    (-136., -132.),
    (-136., -112.),
    (-104., -112.),
    (-104., -92.),
    (-116., -92.),
    (-116., -78.),
];
pub const BRANCHES: [((f32, f32), (f32, f32)); 4] = [
    ((-136., -122.), (-148., -122.)),
    ((-120., -112.), (-120., -125.)),
    ((-104., -100.), (-94., -100.)),
    ((-116., -87.), (-134., -87.)),
];
pub fn underground(p: Vec3) -> bool {
    p.y < -0.5 && (-152. ..=-90.).contains(&p.x) && (-150. ..=-73.).contains(&p.z)
}
pub fn passable(p: Vec3, r: f32) -> bool {
    let segments = ROUTE.windows(2).map(|v| (v[0], v[1])).chain(BRANCHES);
    segments.into_iter().any(|(a, b)| {
        let width = 2.5 - r;
        p.x >= a.0.min(b.0) - width
            && p.x <= a.0.max(b.0) + width
            && p.z >= a.1.min(b.1) - width
            && p.z <= a.1.max(b.1) + width
    })
}
pub fn mesh(m: &mut Mesh, lod: u8) {
    // Concertina wire on the fictional camp perimeter; static LOD geometry.
    for edge in [-220., -100.] {
        for i in 0..28 {
            let segments = if lod == 0 { 16 } else { 4 };
            for j in 0..segments {
                let phase = j as f32 / segments as f32 * std::f32::consts::TAU;
                let next = (j + 1) as f32 / segments as f32 * std::f32::consts::TAU;
                let a = [
                    -168. + i as f32 * 5. + j as f32 * 5. / segments as f32,
                    6.4 + phase.sin() * 0.45,
                    edge + phase.cos() * 0.45,
                ];
                let b = [
                    -168. + i as f32 * 5. + (j + 1) as f32 * 5. / segments as f32,
                    6.4 + next.sin() * 0.45,
                    edge + next.cos() * 0.45,
                ];
                m.quad(
                    [a, b, [b[0], b[1] + 0.04, b[2]], [a[0], a[1] + 0.04, a[2]]],
                    [0.3, 0.32, 0.29],
                    4,
                    0.,
                );
                if lod == 0 && j % 4 == 0 {
                    m.cuboid(a, [0.22, 0.025, 0.025], [0.22, 0.24, 0.21], 4, 0.);
                }
            }
        }
    }

    for z in [-145., -78.] {
        m.cuboid(
            [-116., 0.08, z],
            [2.4, 0.16, 2.4],
            [0.16, 0.23, 0.26],
            4,
            0.,
        );
    }
}
pub fn sewer_mesh() -> Mesh {
    let mut mesh = Mesh::default();
    let m = &mut mesh;
    for x in (-151..-90).step_by(2) {
        for z in (-149..-73).step_by(2) {
            let p = Vec3::new(x as f32, -3., z as f32);
            if !passable(p, 0.) {
                continue;
            }
            m.cuboid([p.x, -4.1, p.z], [2., 0.2, 2.], [0.25, 0.31, 0.3], 1, 0.);
            if p.distance(Vec3::new(-116., -3., -145.)) > 2.
                && p.distance(Vec3::new(-116., -3., -78.)) > 2.
            {
                m.cuboid([p.x, -0.2, p.z], [2., 0.2, 2.], [0.25, 0.3, 0.3], 1, 0.);
            }
            for (dx, dz) in [(2., 0.), (-2., 0.), (0., 2.), (0., -2.)] {
                if !passable(Vec3::new(p.x + dx, -3., p.z + dz), 0.) {
                    m.cuboid(
                        [p.x + dx / 2., -2., p.z + dz / 2.],
                        if dx == 0. {
                            [2., 4., 0.15]
                        } else {
                            [0.15, 4., 2.]
                        },
                        [0.3, 0.38, 0.34],
                        2,
                        0.,
                    );
                }
            }
        }
    }
    for z in [-145., -78.] {
        m.cuboid(
            [-116., 0.08, z],
            [2.4, 0.16, 2.4],
            [0.16, 0.23, 0.26],
            4,
            0.,
        );
        for x in [-117., -115.] {
            m.cuboid([x, -2., z], [0.12, 4., 0.12], [0.75, 0.65, 0.25], 4, 0.);
        }
        for i in 0..10 {
            m.cuboid(
                [-116., -3.8 + i as f32 * 0.4, z],
                [2., 0.12, 0.12],
                [0.75, 0.65, 0.25],
                4,
                0.,
            );
        }
    }
    for z in (-140..-80).step_by(10) {
        m.cuboid(
            [-116., -0.4, z as f32],
            [1., 0.12, 0.5],
            [0.75, 0.95, 0.8],
            9,
            1.,
        );
    }
    mesh
}

pub const CHARGE: Vec3 = Vec3 {
    x: -140.,
    y: 1.,
    z: -135.,
};
pub const WEAK: Vec3 = Vec3 {
    x: -170.,
    y: 1.,
    z: -110.,
};
pub fn barrier(p: Vec3, r: f32, breached: bool) -> bool {
    !breached
        && p.y + 1.5 > 0.
        && p.y - 0.8 < 6.
        && (p.x + 170.).abs() < 0.5 + r
        && (-118. - r..-102. + r).contains(&p.z)
}
