use super::*;
impl Building {
    pub(super) fn power_plant_interior(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        for i in 0..4 {
            m.cuboid(
                [x - 15. + i as f32 * 10., 2., z + 8.],
                [4., 4., 5.],
                [0.24, 0.27, 0.3],
                4,
                0.,
            );
            m.cuboid(
                [x - 15. + i as f32 * 10., 3., z + 5.4],
                [2., 1., 0.1],
                [0.15, 0.85, 0.6],
                9,
                0.5,
            );
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add(
        "Power Plant",
        720.,
        -170.,
        [44., 9., 36.],
        Style::Warehouse,
        true,
    );
}
