use super::*;
impl Building {
    pub(super) fn jewelry_interior(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        for y in [4., 8., 12.] {
            m.cuboid([x, y, z], [28., 0.12, 28.], [0.1, 0.2, 0.28], 1, 0.);
        }
        for i in 0..6 {
            m.cuboid(
                [x - 9. + i as f32 * 3.5, 4.8, z + 3.],
                [2., 1.6, 2.],
                [0.2, 0.35, 0.4],
                4,
                0.,
            );
            m.cuboid(
                [x - 9. + i as f32 * 3.5, 5.8, z + 3.],
                [0.7, 0.7, 0.7],
                [0.15, 0.9, 1.],
                9,
                0.5,
            );
        }
    }
}

impl Building {
    pub(super) fn jewelry_details_15(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;

        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 6., 1.3, z],
                [3., 0.4, 5.],
                [0.5, 0.8, 0.86],
                3,
                0.,
            );
            for i in 0..4 {
                m.cuboid(
                    [x + side * 6., 1.65, z - 1.5 + i as f32],
                    [0.4, 0.3, 0.4],
                    [0.85, 0.65, 0.25],
                    4,
                    0.,
                );
            }
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Jewelry", 260., 190., [28., 12., 28.], Style::Glass, true);
}
