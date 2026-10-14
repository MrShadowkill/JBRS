use engine_core::Mesh;
/// Cross sections give each body a distinct silhouette without curved-box tessellation.
pub fn mesh(kind: usize) -> Mesh {
    let mut m = Mesh::default();
    let (length, roof, cabin_z, width) = match kind {
        1 => (3.65, 1.35, -0.25, 1.0),
        3 => (4.5, 1.15, -0.25, 1.03),
        4 => (5.1, 1.5, 0.65, 1.05),
        6 => (4.55, 1.0, -0.15, 1.14),
        5 => (4.8, 0.95, -0.4, 1.15),
        _ => (4.5, 1.22, -0.35, 1.07),
    };
    let l = length * 0.5;
    sections(
        &mut m,
        &[
            (-l, width * 0.86, 0.1, 0.50),
            (-l + 0.4, width, 0., 0.72),
            (0., width, 0., 0.72),
            (l - 0.5, width * 0.97, 0.06, 0.58),
            (l, width * 0.78, 0.18, 0.42),
        ],
        [1., 1., 1.],
    );
    let cabin_length = if kind == 4 { 1.55 } else { 2.35 };
    sections(
        &mut m,
        &[
            (cabin_z - cabin_length * 0.5, 0.9, 0.65, 0.70),
            (cabin_z - cabin_length * 0.25, 0.75, 0.65, roof),
            (cabin_z + cabin_length * 0.24, 0.72, 0.65, roof),
            (cabin_z + cabin_length * 0.5, 0.88, 0.6, 0.68),
        ],
        [0.12, 0.17, 0.21],
    );
    m.cuboid(
        [0., roof + 0.015, cabin_z],
        [1.45, 0.07, cabin_length * 0.49],
        [1., 1., 1.],
        1,
        0.,
    );
    if kind == 4 {
        // Open pickup bed with separate rails and tailgate.
        m.cuboid(
            [0., 0.73, -1.25],
            [1.8, 0.08, 1.6],
            [0.12, 0.13, 0.14],
            1,
            0.,
        );
        for side in [-1., 1.] {
            m.cuboid(
                [side * 0.96, 0.88, -1.25],
                [0.13, 0.4, 1.8],
                [1., 1., 1.],
                1,
                0.,
            );
        }
    }
    if kind == 6 {
        for side in [-1., 1.] {
            m.cuboid(
                [side * 1.08, 0.5, -0.5],
                [0.12, 0.4, 1.2],
                [0.08, 0.1, 0.12],
                1,
                0.,
            );
        }
        m.cuboid(
            [0., 0.33, 2.23],
            [0.5, 0.4, 0.12],
            [0.06, 0.07, 0.08],
            1,
            0.,
        );
    }
    if kind == 5 {
        m.cuboid([0., 0.98, -2.], [2.25, 0.1, 0.4], [0.15, 0.17, 0.18], 1, 0.);
        for side in [-1., 1.] {
            m.cuboid(
                [side * 0.7, 0.78, -2.],
                [0.08, 0.4, 0.12],
                [0.2, 0.2, 0.2],
                1,
                0.,
            );
        }
    }
    m
}
fn sections(m: &mut Mesh, rings: &[(f32, f32, f32, f32)], color: [f32; 3]) {
    let ring = |(z, w, lo, hi): (f32, f32, f32, f32)| {
        [
            [-w * 0.8, lo, z],
            [-w, lo + 0.12, z],
            [-w, hi - 0.12, z],
            [-w * 0.8, hi, z],
            [w * 0.8, hi, z],
            [w, hi - 0.12, z],
            [w, lo + 0.12, z],
            [w * 0.8, lo, z],
        ]
    };
    for pair in rings.windows(2) {
        let a = ring(pair[0]);
        let b = ring(pair[1]);
        for i in 0..8 {
            let j = (i + 1) % 8;
            let shade = [0.6, 0.7, 0.85, 1., 0.9, 0.8, 0.7, 0.6][i];
            m.quad([a[i], a[j], b[j], b[i]], color.map(|v| v * shade), 1, 0.);
        }
    }
    for end in [0, rings.len() - 1] {
        let a = ring(rings[end]);
        for i in 1..7 {
            m.triangle([a[0], a[i], a[i + 1]], color.map(|v| v * 0.8), 1, 0.);
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn body_budget_and_distinct_profiles() {
        let meshes: Vec<_> = (0..7).map(super::mesh).collect();
        for m in &meshes {
            assert!(m.vertices.len() / 3 < 400);
            assert!(m
                .vertices
                .iter()
                .all(|v| v.position.iter().all(|x| x.is_finite())));
        }
        assert_ne!(
            meshes[1].vertices[0].position,
            meshes[4].vertices[0].position
        );
    }
}
