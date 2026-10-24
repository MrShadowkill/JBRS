use super::*;

impl Building {
    pub(super) fn gym_details_6(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;

        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 5., 0.6, z],
                [1.3, 1.2, 4.],
                [0.12, 0.16, 0.17],
                4,
                0.,
            );
            for zz in [-1.7, 1.7] {
                m.cylinder(
                    [x + side * 5., 0.9, z + zz],
                    0.48,
                    0.1,
                    [0.2, 0.22, 0.22],
                    4,
                    16,
                );
            }
            m.cuboid(
                [x + side * 5., 1.7, z + 6.],
                [0.7, 2.2, 0.7],
                [0.65, 0.14, 0.12],
                10,
                0.,
            );
            m.cuboid(
                [x + side * 5., 3., z + 6.],
                [0.08, 1., 0.08],
                [0.25, 0.28, 0.28],
                4,
                0.,
            );
        }
        m.cuboid(
            [x, 0.06, z - 5.],
            [7., 0.12, 5.],
            [0.17, 0.28, 0.32],
            10,
            0.,
        );
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Gym", -260., 340., [32., 6., 28.], Style::Warehouse, true);
}
