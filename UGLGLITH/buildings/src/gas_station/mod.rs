use super::*;
impl Building {
    pub(super) fn gas_station_interior(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 8., 1., z],
                [4., 2., 3.],
                [0.65, 0.65, 0.55],
                1,
                0.,
            );
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add(
        "Gas Station",
        -650.,
        40.,
        [30., 6., 24.],
        Style::Civic,
        true,
    );
}
