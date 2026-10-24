use super::*;

impl Building {
    pub(super) fn prison_details_0(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        let d = self.size.z;

        for side in [-1., 1.] {
            for i in 0..4 {
                let zz = z - d / 2. + 5. + i as f32 * 10.;
                let xx = x + side * 11.;
                for dx in [-0.85, 0.85] {
                    for dz in [-1.6, 1.6] {
                        m.cuboid(
                            [xx + dx, 1., zz - 1. + dz],
                            [0.08, 2., 0.08],
                            [0.23, 0.27, 0.26],
                            4,
                            0.,
                        );
                    }
                }
                m.cuboid(
                    [xx, 1.55, zz - 1.],
                    [2., 0.12, 3.5],
                    [0.28, 0.34, 0.34],
                    4,
                    0.,
                );
                m.cuboid(
                    [xx, 1.69, zz - 1.],
                    [1.9, 0.18, 3.35],
                    [0.42, 0.43, 0.34],
                    10,
                    0.,
                );
                m.cuboid(
                    [xx, 1.82, zz - 2.2],
                    [1.3, 0.16, 0.5],
                    [0.68, 0.66, 0.56],
                    10,
                    0.,
                );
            }
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add(
        "Cell Block",
        -145.,
        -183.,
        [28., 6., 44.],
        Style::Brick,
        true,
    );
}
