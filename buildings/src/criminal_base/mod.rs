use super::*;

impl Building {
    pub(super) fn criminal_base_details_5(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        let d = self.size.z;

        m.cuboid(
            [x - 5., 0.55, z + 4.],
            [5., 1.1, 2.],
            [0.28, 0.24, 0.2],
            10,
            0.,
        );
        m.cuboid(
            [x - 5., 1.3, z + 4.8],
            [5., 1.5, 0.35],
            [0.3, 0.25, 0.19],
            10,
            0.,
        );
        for i in 0..3 {
            m.cuboid(
                [x + 5., 0.45, z - 5. + i as f32 * 2.],
                [2., 0.9, 1.5],
                [0.22, 0.28, 0.15],
                10,
                0.,
            );
        }
        m.cuboid(
            [x, 2.2, z + d / 2. - 0.5],
            [7., 2.4, 0.1],
            [0.24, 0.35, 0.3],
            6,
            0.,
        );
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Safehouse", -335., 320., [28., 5., 24.], Style::Brick, true);
}

pub fn place_1(lib: &mut Library) {
    lib.add(
        "Criminal Base",
        -290.,
        100.,
        [34., 7., 30.],
        Style::Warehouse,
        true,
    );
}
