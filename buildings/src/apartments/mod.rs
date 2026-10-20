use super::*;

#[derive(Clone, Copy, Debug)]
pub struct InteractionPoints {
    pub entry_door: Vec3,
    pub room_door: Vec3,
    pub light_switch: Vec3,
    pub fridge: Vec3,
    pub shelves: Vec3,
    pub wardrobe: Vec3,
    pub bed: Vec3,
}

pub fn interaction_points(b: &Building, floor: usize, side: f32) -> InteractionPoints {
    let variant = (b.id * 13 + floor * 7) % 32;
    let z_offset = (variant % 4) as f32 - 1.5;
    let unit_x = b.p.x + side * (b.size.x / 2.0 - 7.0);
    let y = b.p.y + floor as f32 * 4.0;
    InteractionPoints {
        entry_door: Vec3::new(b.p.x + side * 3.0, y + 1.5, b.p.z),
        room_door: Vec3::new(b.p.x + side * 6.8, y + 1.5, b.p.z),
        light_switch: Vec3::new(b.p.x + side * 4.1, y + 1.3, b.p.z - 2.2),
        fridge: Vec3::new(b.p.x + side * (b.size.x / 2.0 - 1.2), y + 1.1, b.p.z + 2.0),
        shelves: Vec3::new(unit_x - side * 4.0, y + 1.0, b.p.z + z_offset - 5.0),
        wardrobe: Vec3::new(unit_x + side * 3.0, y + 1.4, b.p.z + z_offset + 6.0),
        bed: Vec3::new(unit_x, y + 1.0, b.p.z + z_offset - 7.0),
    }
}

impl Building {
    pub(super) fn apartments_interior(&self, m: &mut Mesh, selected: Option<usize>) {
        let x = self.p.x;
        let z = self.p.z;
        let count = (self.size.y / 4.) as usize;
        let first = selected.unwrap_or(0).min(count.saturating_sub(1));
        let end = if selected.is_some() {
            (first + 1).min(count)
        } else {
            count
        };
        for floor in first..end {
            let y = floor as f32 * 4.;
            let variant = (self.id * 13 + floor * 7) % 32;
            for side in [-1., 1.] {
                for offset in [-1., 1.] {
                    m.cuboid(
                        [x + side * 3., y + 1.6, z + offset * 7.5],
                        [0.18, 3.2, 11.],
                        [0.72, 0.69, 0.62],
                        1,
                        0.,
                    );
                }
                for fixture in apartment_rooms::room_partitions(self, floor, side) {
                    m.cuboid(
                        [fixture.p.x, fixture.p.y, fixture.p.z],
                        [fixture.size.x, fixture.size.y, fixture.size.z],
                        [0.66, 0.63, 0.57],
                        1,
                        0.,
                    );
                }
                for fixture in apartment_rooms::room_fixtures(self, floor, side) {
                    m.cuboid(
                        fixture.position,
                        fixture.size,
                        fixture.color,
                        fixture.material,
                        0.,
                    );
                }
                // Apartment entry door frame; the animated leaf is drawn by the game.
                for z_offset in [-1.15, 1.15] {
                    m.cuboid(
                        [x + side * 3., y + 1.5, z + z_offset],
                        [0.22, 3., 0.18],
                        [0.39, 0.27, 0.18],
                        10,
                        0.,
                    );
                }
                m.cuboid(
                    [x + side * 3., y + 2.9, z],
                    [0.22, 0.2, 2.45],
                    [0.39, 0.27, 0.18],
                    10,
                    0.,
                );
            }
            m.cuboid(
                [x, y, z],
                [self.size.x, 0.12, self.size.z],
                [0.5, 0.48, 0.43],
                1,
                0.,
            );
            // Warm ceiling fixtures use material emission, which the world
            // shaders enable only after daylight fades.
            for side in [-1., 1.] {
                m.cuboid(
                    [x + side * 6.8, y + 3.24, z - 1.5],
                    [0.42, 0.07, 1.25],
                    [1.0, 0.78, 0.48],
                    9,
                    0.0,
                );
            }
            // Put the lift at the far end of the corridor, opposite the street door.
            for side in [-1., 1.] {
                m.cuboid(
                    [x + side * 1.9, y + 1.5, z + 9.7],
                    [0.16, 3., 3.8],
                    [0.24, 0.28, 0.32],
                    4,
                    0.,
                );
                m.cuboid(
                    [x + side * 1.45, y + 1.5, z + 7.8],
                    [0.8, 3., 0.15],
                    [0.28, 0.31, 0.35],
                    4,
                    0.,
                );
            }
            m.cuboid(
                [x, y + 1.5, z + 11.6],
                [3.8, 3., 0.16],
                [0.24, 0.28, 0.32],
                4,
                0.,
            );
            m.cuboid(
                [x + 2., y + 1.4, z + 7.1],
                [0.18, 0.35, 0.25],
                [0.15, 0.7, 0.9],
                9,
                0.2,
            );
            m.price([x - 0.5, y + 3.2, z + 7.7], 0.15, floor as u32);
            if floor == 0 {
                // Lobby: mailboxes and benches, no apartments at street level.
                for i in 0..8 {
                    m.cuboid(
                        [x - 7. + i as f32 * 0.65, y + 1.6, z - 11.],
                        [0.6, 0.7, 0.25],
                        [0.3, 0.38, 0.44],
                        4,
                        0.,
                    );
                }
                for side in [-1., 1.] {
                    m.cuboid(
                        [x + side * 7., y + 0.5, z],
                        [3., 0.3, 1.],
                        [0.45, 0.28, 0.14],
                        2,
                        0.,
                    );
                }
                continue;
            }
            for f in apartment_rooms::fixtures(self, floor) {
                m.cuboid(f.position, f.size, f.color, f.material, 0.);
            }
            for side in [-1., 1.] {
                let rx = x + side * (self.size.x / 2. - 7.);
                let rz = z + (variant % 4) as f32 - 1.5;
                m.cuboid(
                    [rx, y + 0.5, rz + 5.],
                    [4., 1., 2.],
                    [0.25 + (variant % 3) as f32 * 0.15, 0.4, 0.55],
                    1,
                    0.,
                );
                m.cuboid(
                    [rx, y + 0.6, rz - 5.],
                    [3., 1.2, 5.],
                    [0.8, 0.78, 0.67],
                    1,
                    0.,
                );
                m.cuboid([rx, y + 0.9, rz], [2.5, 0.2, 2.], [0.45, 0.3, 0.18], 2, 0.);
                m.cuboid(
                    [rx + 3., y + 1.2, rz + 6.],
                    [0.15, 2.4, 3.],
                    [0.1, 0.12, 0.16],
                    4,
                    0.,
                );
                // Bed pillows and headboard make the bedroom readable at LOW.
                m.cuboid(
                    [rx, y + 1.25, rz - 6.5],
                    [2.5, 0.25, 0.8],
                    [0.94, 0.93, 0.88],
                    1,
                    0.,
                );
                m.cuboid(
                    [rx, y + 1., rz - 7.5],
                    [3., 2., 0.15],
                    [0.36, 0.23, 0.14],
                    2,
                    0.,
                );
                m.cuboid(
                    [rx + 2.4, y + 1.4, rz + 6.],
                    [1.4, 2.8, 2.],
                    [0.4, 0.27, 0.15],
                    2,
                    0.,
                );
                for i in 0..3 {
                    m.cuboid(
                        [rx - 4., y + 0.4 + i as f32 * 0.6, rz - 5.],
                        [1., 0.08, 3.],
                        [0.5, 0.32, 0.2],
                        2,
                        0.,
                    );
                }
                m.cuboid(
                    [rx, y + 2.5, z + 13.5],
                    [3., 1.8, 0.06],
                    [0.35, 0.2 + (variant % 5) as f32 * 0.1, 0.55],
                    1,
                    0.,
                );
            }
        }
    }
}

/// Search positions match generated furniture; one deterministic stash per apartment.
pub fn search_target(b: &Building, p: Vec3) -> Option<(u64, bool, &'static str)> {
    let floor = b.rendered_floor(p);
    if floor == 0 {
        return None;
    }
    let variant = (b.id * 13 + floor * 7) % 32;
    let mut candidates = Vec::new();
    for (side_id, side) in [-1., 1.].into_iter().enumerate() {
        let rx = b.p.x + side * (b.size.x / 2. - 7.);
        let rz = b.p.z + (variant % 4) as f32 - 1.5;
        for (item, (dx, dz, label)) in [
            (0., 5., "SOFA"),
            (0., -5., "BED"),
            (0., 0., "TABLE"),
            (2.4, 6., "WARDROBE"),
            (-4., -5., "SHELVES"),
        ]
        .into_iter()
        .enumerate()
        {
            let q = Vec3::new(rx + dx, b.p.y + floor as f32 * 4. + 1., rz + dz);
            if p.distance(q) < 2.8 {
                let key =
                    ((b.id as u64 * 64 + floor as u64) * 2 + side_id as u64) * 8 + item as u64;
                candidates.push((
                    p.distance(q),
                    (key, (variant + side_id * 3) % 5 == item, label),
                ));
            }
        }
    }
    candidates
        .into_iter()
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|v| v.1)
}

pub(super) fn furniture_colliders(b: &Building) -> Vec<Collider> {
    let mut out = Vec::new();
    for floor in 1..(b.size.y / 4.) as usize {
        out.extend(apartment_rooms::colliders(b, floor));
        let variant = (b.id * 13 + floor * 7) % 32;
        for side in [-1., 1.] {
            out.extend(apartment_rooms::room_partitions(b, floor, side));
            out.extend(
                apartment_rooms::room_fixtures(b, floor, side)
                    .into_iter()
                    .map(|f| Collider {
                        p: Vec3::new(f.position[0], f.position[1], f.position[2]),
                        size: Vec3::new(f.size[0], f.size[1], f.size[2]),
                        movable: false,
                        breakable: false,
                    }),
            );
            let x = b.p.x + side * (b.size.x / 2. - 7.);
            let z = b.p.z + (variant % 4) as f32 - 1.5;
            for (dx, y, dz, sx, sy, sz) in [
                (0., 0.5, 5., 4., 1., 2.),
                (0., 0.6, -5., 3., 1.2, 5.),
                (0., 0.9, 0., 2.5, 0.2, 2.),
                (2.4, 1.4, 6., 1.4, 2.8, 2.),
                (-4., 1., -5., 1., 2., 3.),
            ] {
                out.push(Collider {
                    p: Vec3::new(x + dx, floor as f32 * 4. + y, z + dz),
                    size: Vec3::new(sx, sy, sz),
                    movable: false,
                    breakable: false,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn building() -> Building {
        Building {
            id: 9,
            name: "Office",
            p: Vec3::new(0., 10., 0.),
            size: Vec3::new(30., 64., 30.),
            style: Style::Glass,
            interior: true,
        }
    }
    #[test]
    fn only_active_floor_is_generated() {
        let b = building();
        let m = b.interior_mesh_at(2);
        assert!(m
            .vertices
            .iter()
            .all(|v| v.position[1] >= 17.9 && v.position[1] <= 22.1));
        assert!(m.vertices.len() < 5000);
        assert!(b.interior_mesh().vertices.len() > m.vertices.len() * 5);
        assert!(b.interior_mesh_at(0).vertices.len() < m.vertices.len());
    }
    #[test]
    fn each_apartment_has_one_stash_and_lobby_none() {
        let b = building();
        assert!(search_target(&b, Vec3::new(8., 11., 0.)).is_none());
        for floor in 1..16 {
            let variant = (b.id * 13 + floor * 7) % 32;
            for side in [-1., 1.] {
                let mut found = 0;
                for (dx, dz) in [(0., 5.), (0., -5.), (0., 0.), (2.4, 6.), (-4., -5.)] {
                    let p = Vec3::new(
                        side * 8. + dx,
                        11. + floor as f32 * 4.,
                        (variant % 4) as f32 - 1.5 + dz,
                    );
                    if search_target(&b, p).is_some_and(|(_, stash, _)| stash) {
                        found += 1;
                    }
                }
                assert_eq!(found, 1);
            }
        }
    }
}
