use super::*;

impl Building {
    pub(super) fn donut_shop_details_12(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        let w = self.size.x;

        for side in [-1., 1.] {
            m.cuboid(
                [x + side * w * 0.32, 1.18, z],
                [2.2, 0.15, 4.],
                [0.93, 0.75, 0.57],
                1,
                0.,
            );
        }
    }
}

impl Building {
    pub(super) fn donut_shop_details_3(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;

        for i in 0..6 {
            m.torus(
                [
                    x + 5. + (i % 3) as f32 * 0.6,
                    1.27,
                    z + (i / 3) as f32 * 0.8,
                ],
                [0.18, 0.07],
                [0.87, 0.49, 0.57],
                16,
            );
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Donut Shop", 110., 240., [28., 6., 28.], Style::Neon, true);
}
