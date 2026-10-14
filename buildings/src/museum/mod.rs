use super::*;
impl Building {
    pub(super) fn museum_interior(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        for i in 0..5 {
            m.cuboid(
                [x - 14. + i as f32 * 7., 2.5, z + 17.],
                [4., 3., 0.06],
                [0.2 + i as f32 * 0.1, 0.35, 0.55],
                1,
                0.,
            );
            museum_sphere(
                m,
                [x - 14. + i as f32 * 7., 6., z],
                1. + i as f32 * 0.12,
                [0.3 + i as f32 * 0.1, 0.35, 0.6],
            );
            m.torus(
                [x - 14. + i as f32 * 7., 6., z],
                [1. + i as f32 * 0.12, 0.6],
                [0.4, 0.2 + i as f32 * 0.1, 0.6],
                16,
            );
        }
        m.cylinder([x, 0.03, z], 8., 0.05, [0.55, 0.08, 0.07], 1, 24);
        let bone = [0.87, 0.83, 0.68];
        museum_bone(m, [x, 3., z - 3.], [x, 3., z + 3.], 0.16, bone);
        for i in 0..8 {
            let rz = z - 2.5 + i as f32 * 0.7;
            for side in [-1., 1.] {
                for j in 0..8 {
                    let a = j as f32 / 8. * std::f32::consts::PI;
                    let b = (j + 1) as f32 / 8. * std::f32::consts::PI;
                    museum_bone(
                        m,
                        [x + side * a.sin() * 1.4, 3. - (1. - a.cos()) * 0.65, rz],
                        [x + side * b.sin() * 1.4, 3. - (1. - b.cos()) * 0.65, rz],
                        0.09,
                        bone,
                    );
                }
            }
        }
        for side in [-1., 1.] {
            for dz in [-2., 2.] {
                museum_bone(
                    m,
                    [x + side * 0.7, 2.8, z + dz],
                    [x + side * 1.2, 1.4, z + dz + 0.3],
                    0.17,
                    bone,
                );
                museum_bone(
                    m,
                    [x + side * 1.2, 1.4, z + dz + 0.3],
                    [x + side * 1.1, 0.35, z + dz - 0.4],
                    0.14,
                    bone,
                );
                m.cuboid(
                    [x + side * 1.1, 0.2, z + dz - 0.7],
                    [0.5, 0.25, 0.9],
                    bone,
                    1,
                    0.,
                );
            }
        }
        museum_bone(m, [x, 3., z - 3.], [x, 3.8, z - 4.], 0.2, bone);
        m.cuboid([x, 3.8, z - 5.], [1.7, 0.8, 2.5], bone, 1, 0.);
        m.cuboid([x, 3.25, z - 5.], [1.4, 0.18, 2.3], bone, 1, 0.);
        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 0.86, 3.95, z - 4.5],
                [0.04, 0.35, 0.5],
                [0.08, 0.08, 0.07],
                1,
                0.,
            );
            for i in 0..8 {
                m.cuboid(
                    [x + side * 0.65, 3.5, z - 5.9 + i as f32 * 0.25],
                    [0.12, 0.3, 0.12],
                    bone,
                    1,
                    0.,
                );
            }
        }
        for i in 0..5 {
            museum_bone(
                m,
                [x, 3. - i as f32 * 0.3, z + 3. + i as f32],
                [x, 2.7 - i as f32 * 0.3, z + 4. + i as f32],
                0.15 - i as f32 * 0.02,
                bone,
            );
        }
        m.cylinder([x + 13., 0.1, z + 4.], 0.65, 2.2, [0.76, 0.72, 0.6], 1, 16);
        museum_sphere(m, [x + 13., 2.65, z + 4.], 0.5, bone);
        for i in 0..9 {
            m.torus(
                [x + 13., 0.3 + i as f32 * 0.22, z + 4.],
                [0.65, 0.035],
                [0.9, 0.85, 0.68],
                16,
            );
        }
        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 14., 0.6, z - 4.],
                [4., 1.2, 2.],
                [0.2, 0.25, 0.3],
                4,
                0.,
            );
            m.cuboid(
                [x + side * 14., 1.5, z - 4.],
                [4., 0.06, 2.],
                [0.2, 0.65, 0.85],
                3,
                0.,
            );
            for i in 0..3 {
                museum_gem(m, [x + side * 14. - 1. + i as f32, 1.8, z - 4.], 0.3);
            }
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Museum", -200., -630., [42., 10., 36.], Style::Brick, true);
}
