use super::*;
impl Building {
    pub(super) fn bank_interior(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        m.cuboid([x, 2., z + 10.], [5., 4., 0.6], [0.23, 0.27, 0.3], 4, 0.);
        m.torus([x, 2., z + 9.6], [1., 0.15], [0.65, 0.65, 0.68], 16);
        for i in 0..5 {
            m.cuboid(
                [x - 4. + i as f32 * 2., 0.4, z - 3.],
                [1.2, 0.3, 0.8],
                [0.15, 0.7, 0.28],
                1,
                0.,
            );
        }
    }
}

impl Building {
    pub(super) fn bank_details_14(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        let d = self.size.z;

        m.cuboid(
            [x, 2., z + d / 2. - 2.],
            [9., 4., 0.6],
            [0.4, 0.45, 0.48],
            4,
            0.,
        );
        m.cylinder(
            [x, 1.8, z + d / 2. - 2.4],
            1.2,
            0.25,
            [0.72, 0.72, 0.7],
            4,
            12,
        );
    }
}

impl Building {
    pub(super) fn bank_details_13(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;

        for dx in [-10., -6., 6., 10.] {
            m.cuboid(
                [x + dx, 0.6, z + 8.],
                [2.8, 1.2, 1.1],
                [0.4, 0.34, 0.24],
                10,
                0.,
            );
            m.cuboid(
                [x + dx, 2., z + 8.],
                [2.8, 1.3, 0.07],
                [0.47, 0.62, 0.66],
                3,
                0.,
            );
            m.cuboid(
                [x + dx, 1.35, z - 8.],
                [1.5, 0.1, 1.5],
                [0.36, 0.4, 0.45],
                4,
                0.,
            );
            m.cuboid(
                [x + dx, 0.65, z - 8.],
                [0.12, 1.3, 0.12],
                [0.3, 0.34, 0.35],
                4,
                0.,
            );
        }
        for side in [-1., 1.] {
            for i in 0..3 {
                m.cuboid(
                    [x + side * 12., 0.5, z - 6. + i as f32 * 2.],
                    [1.4, 1., 1.2],
                    [0.24, 0.32, 0.34],
                    10,
                    0.,
                );
            }
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Bank", 320., 35., [40., 8., 40.], Style::Civic, true);
}
