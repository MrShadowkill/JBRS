use super::*;

impl Building {
    pub(super) fn factory_details_8(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;

        for side in [-1., 1.] {
            for i in 0..3 {
                m.cuboid(
                    [x + side * 14., 1.5, z - 8. + i as f32 * 7.],
                    [6., 3., 4.],
                    [0.25, 0.35, 0.4],
                    4,
                    0.,
                );
                m.cylinder(
                    [x + side * 14., 3., z - 8. + i as f32 * 7.],
                    0.7,
                    2.,
                    [0.4, 0.43, 0.42],
                    4,
                    16,
                );
            }
        }
        m.cuboid([x, 0.4, z + 10.], [8., 0.8, 3.], [0.24, 0.27, 0.29], 4, 0.);
    }
}

pub fn place_0(lib: &mut Library, x: f32) {
    lib.add(
        "Factory",
        x,
        220.,
        [48., 12., 44.],
        Style::Warehouse,
        x >= 640.,
    );
}
