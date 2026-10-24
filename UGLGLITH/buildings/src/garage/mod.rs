use super::*;

impl Building {
    pub(super) fn garage_details_4(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;

        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 5., 1.8, z],
                [0.6, 3.6, 0.6],
                [0.67, 0.15, 0.1],
                4,
                0.,
            );
            m.cuboid(
                [x + side * 3.2, 0.2, z],
                [4., 0.4, 5.],
                [0.23, 0.25, 0.25],
                4,
                0.,
            );
        }
        m.cuboid(
            [x - 10., 1., z + 8.],
            [5., 2., 1.5],
            [0.62, 0.12, 0.1],
            4,
            0.,
        );
        for i in 0..4 {
            m.cuboid(
                [x - 10., 0.4 + i as f32 * 0.4, z + 7.2],
                [4.7, 0.08, 0.06],
                [0.75, 0.76, 0.71],
                4,
                0.,
            );
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add(
        "Desert Garage",
        700.,
        -330.,
        [30., 7., 30.],
        Style::Warehouse,
        true,
    );
}

pub fn place_1(lib: &mut Library) {
    lib.add("Garage", 430., -45., [34., 6., 28.], Style::Warehouse, true);
}
