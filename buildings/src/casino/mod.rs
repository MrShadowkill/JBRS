use super::*;
impl Building {
    pub(super) fn casino_interior(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        m.cuboid(
            [x, 8., z],
            [self.size.x, 0.12, self.size.z],
            [0.4, 0.42, 0.45],
            1,
            0.,
        );
        for (cx, sx) in [(-5.75, 22.5), (12.75, 8.5)] {
            m.cuboid(
                [x + cx, 4., z],
                [sx, 0.12, self.size.z],
                [0.32, 0.34, 0.37],
                1,
                0.,
            );
        }
        m.cuboid(
            [x + 7., 4., z - 14.45],
            [3., 0.12, 5.1],
            [0.32, 0.34, 0.37],
            1,
            0.,
        );
        m.cuboid(
            [x + 7., 4., z + 12.55],
            [3., 0.12, 8.9],
            [0.32, 0.34, 0.37],
            1,
            0.,
        );
        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 9., 5., z + 5.],
                [4., 0.2, 3.],
                [0.08, 0.4, 0.24],
                1,
                0.,
            );
            for card in 0..5 {
                m.cuboid(
                    [x + side * 9. - 1. + card as f32 * 0.5, 5.15, z + 5.],
                    [0.3, 0.03, 0.5],
                    [0.9, 0.9, 0.86],
                    1,
                    0.,
                );
            }
        }
        for i in 0..6 {
            museum_sphere(
                m,
                [x - 7. + i as f32 * 0.65, 1.3, z + 5.],
                0.09,
                [0.8, 0.1 + i as f32 * 0.1, 0.1],
            );
        }
        m.cuboid(
            [x - 12., 5.8, z - 10.],
            [3., 1.5, 0.1],
            [0.1, 0.5, 0.8],
            9,
            0.4,
        );

        for i in 0..6 {
            m.cuboid(
                [x - 12., 1.5, z - 9. + i as f32 * 3.],
                [1.4, 3., 1.4],
                [0.2, 0.12, 0.3],
                4,
                0.,
            );
            m.price([x - 11.2, 2., z - 9. + i as f32 * 3.], 0.1, 777);
        }
        m.cuboid(
            [x - 5., 1., z + 5.],
            [6., 0.3, 3.],
            [0.03, 0.45, 0.28],
            1,
            0.,
        );
        for dx in [-7., -3.] {
            for dz in [4., 6.] {
                m.cuboid(
                    [x + dx, 0.5, z + dz],
                    [0.2, 1., 0.2],
                    [0.4, 0.23, 0.1],
                    2,
                    0.,
                );
            }
        }
        m.cuboid(
            [x + 8., 8., z + 9.],
            [12., 0.2, 10.],
            [0.45, 0.45, 0.5],
            1,
            0.,
        );
        m.cuboid(
            [x + 12., 9., z + 12.],
            [3., 0.2, 2.],
            [0.3, 0.3, 0.3],
            4,
            0.,
        );
        m.cuboid(
            [x + 12., 9.8, z + 12.],
            [1.6, 1.1, 0.1],
            [0.1, 0.7, 0.9],
            9,
            0.4,
        );
    }
}

impl Building {
    pub(super) fn casino_details_10(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        let w = self.size.x;
        let d = self.size.z;

        let start = z - d * 0.35;
        for i in 0..20 {
            let top = (i + 1) as f32 * 0.2;
            m.cuboid(
                [x + 7., top / 2., start + i as f32 + 0.5],
                [3., top, 1.],
                [0.62, 0.62, 0.58],
                1,
                0.,
            );
        }
        m.cuboid(
            [
                x + (4. + w / 2.) / 2.,
                3.9,
                z + (-d * 0.35 + 20. + d / 2.) / 2.,
            ],
            [w / 2. - 4., 0.2, d / 2. - (-d * 0.35 + 20.)],
            [0.64, 0.64, 0.6],
            1,
            0.,
        );
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Casino", 370., 240., [34., 12., 34.], Style::Neon, true);
}
