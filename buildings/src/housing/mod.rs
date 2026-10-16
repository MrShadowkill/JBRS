use super::*;

impl Building {
    pub(super) fn housing_details_7(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        let d = self.size.z;
        if matches!(self.name, "Apartments" | "Penthouse") {
            for f in apartment_rooms::ground_fixtures(self) {
                m.cuboid(f.position, f.size, f.color, f.material, 0.);
            }
        }

        m.cuboid(
            [x - 5., 0.5, z + 5.],
            [4., 1., 2.],
            [0.17, 0.28, 0.4],
            10,
            0.,
        );
        m.cuboid(
            [x - 5., 1.1, z + 5.7],
            [4., 1.4, 0.4],
            [0.2, 0.32, 0.45],
            10,
            0.,
        );
        m.cuboid(
            [x - 5., 2.4, z - d / 2. + 0.5],
            [3.5, 2., 0.12],
            [0.09, 0.11, 0.14],
            4,
            0.,
        );
        m.cuboid(
            [x + 5., 0.45, z + 5.],
            [3., 0.9, 4.],
            [0.6, 0.52, 0.42],
            10,
            0.,
        );
        m.cuboid(
            [x + 5., 1., z + 5.],
            [2.9, 0.2, 3.9],
            [0.83, 0.83, 0.77],
            10,
            0.,
        );
        m.cuboid(
            [x + 5., 1.2, z + 6.4],
            [2., 0.22, 0.8],
            [0.7, 0.75, 0.8],
            10,
            0.,
        );
        m.cuboid(
            [x + 6., 1., z - 6.],
            [4., 2., 1.5],
            [0.55, 0.5, 0.42],
            10,
            0.,
        );
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Penthouse", 350., -50., [28., 8., 24.], Style::Glass, true);
}
