use super::*;

impl Building {
    pub(super) fn grocery_details_11(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        let w = self.size.x;
        let d = self.size.z;

        for side in [-1., 1.] {
            for shelf in 0..3 {
                for row in 0..3 {
                    m.cuboid(
                        [
                            x + side * w * 0.32,
                            0.8 + row as f32 * 0.6,
                            z - d * 0.28 + shelf as f32 * d * 0.25,
                        ],
                        [w * 0.2, 0.1, 2.],
                        [0.7, 0.72, 0.67],
                        4,
                        0.,
                    );
                    for item in 0..5 {
                        m.cuboid(
                            [
                                x + side * w * 0.32 - w * 0.07 + item as f32 * 0.8,
                                1. + row as f32 * 0.6,
                                z - d * 0.28 + shelf as f32 * d * 0.25,
                            ],
                            [0.4, 0.3, 0.45],
                            [0.5, 0.6 - item as f32 * 0.06, 0.2 + row as f32 * 0.15],
                            1,
                            0.,
                        );
                    }
                }
            }
        }
    }
}

impl Building {
    pub(super) fn grocery_details_2(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        let w = self.size.x;
        let d = self.size.z;

        for side in [-1., 1.] {
            for shelf in 0..3 {
                for row in 0..3 {
                    for item in 0..6 {
                        let xx = x + side * w * 0.32 - w * 0.07 + item as f32 * 0.7;
                        let yy = 0.86 + row as f32 * 0.6;
                        let zz = z - d * 0.28 + shelf as f32 * d * 0.25 - 0.65;
                        if item % 2 == 0 {
                            m.cylinder([xx, yy, zz], 0.14, 0.32, [0.68, 0.25, 0.14], 4, 12);
                            m.cylinder([xx, yy + 0.32, zz], 0.145, 0.025, [0.78, 0.8, 0.78], 4, 12);
                        } else {
                            m.cylinder([xx, yy, zz], 0.11, 0.4, [0.34, 0.65, 0.78], 3, 12);
                            m.cylinder([xx, yy + 0.4, zz], 0.06, 0.12, [0.28, 0.48, 0.7], 4, 10);
                            m.cuboid(
                                [xx, yy + 0.18, zz - 0.112],
                                [0.18, 0.14, 0.02],
                                [0.9, 0.93, 0.9],
                                1,
                                0.,
                            );
                        }
                        m.cuboid(
                            [xx, yy - 0.06, zz - 0.41],
                            [0.42, 0.14, 0.03],
                            [0.98, 0.94, 0.73],
                            1,
                            0.,
                        );
                        m.price(
                            [xx - 0.17, yy - 0.015, zz - 0.43],
                            0.015,
                            if item % 2 == 0 { 12 } else { 8 },
                        );
                    }
                }
            }
        }
        for i in 0..3 {
            m.cuboid(
                [x + w / 2. - 1., 1.4, z + 2. + i as f32 * 3.],
                [1.3, 2.8, 2.6],
                [0.78, 0.81, 0.8],
                4,
                0.,
            );
            m.cuboid(
                [x + w / 2. - 1.72, 1.55, z + 2. + i as f32 * 3.],
                [0.06, 2.2, 2.3],
                [0.34, 0.52, 0.57],
                3,
                0.,
            );
        }
    }
}

impl Building {
    pub(super) fn grocery_details_1(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        let w = self.size.x;
        let d = self.size.z;

        m.cuboid(
            [x + 4., 0.55, z + 6.],
            [6., 1.1, 1.4],
            [0.35, 0.28, 0.21],
            10,
            0.,
        );
        m.cuboid(
            [x + 4., 1.15, z + 6.],
            [6.3, 0.15, 1.6],
            [0.77, 0.77, 0.72],
            4,
            0.,
        );
        m.cuboid(
            [x + 3., 1.45, z + 6.],
            [0.8, 0.5, 0.6],
            [0.12, 0.15, 0.16],
            4,
            0.,
        );
        m.cuboid(
            [x + 3., 1.73, z + 5.85],
            [0.65, 0.3, 0.06],
            [0.32, 0.68, 0.62],
            9,
            0.4,
        );
        m.cylinder(
            [x - w / 2. + 1.2, 0.05, z + d / 2. - 2.],
            0.45,
            0.8,
            [0.22, 0.28, 0.27],
            4,
            12,
        );
        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 3.2, 1.8, z - 3.],
                [0.18, 0.35, 0.18],
                [0.9, 0.13, 0.1],
                9,
                0.3,
            );
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Grocery", 210., 240., [28., 6., 28.], Style::Civic, true);
}
