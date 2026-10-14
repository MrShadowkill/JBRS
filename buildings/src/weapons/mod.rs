use super::*;

impl Building {
    pub(super) fn weapons_details_9(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;

        for (kind, price) in [350, 600, 950].iter().enumerate() {
            m.cuboid(
                [x - 6. + kind as f32 * 5., 0.98, z + 4.38],
                [1.1, 0.35, 0.02],
                [0.85, 0.87, 0.82],
                1,
                0.,
            );
            m.price([x - 6.45 + kind as f32 * 5., 1.08, z + 4.36], 0.035, *price);
        }
    }
}

pub fn place_0(lib: &mut Library) {
    lib.add("Weapons", 60., 240., [28., 6., 28.], Style::Brick, true);
}
