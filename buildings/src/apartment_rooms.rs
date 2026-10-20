//! Deterministic apartment plans shared by interior meshes and physics.
use super::{Building, Collider, Mesh, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutKind {
    Studio,
    OneBedroom,
    TwoBedroom,
    ThreeBedroom,
    LongHall,
    Corner,
}

/// Stable address independent of rendering order or generated mesh residency.
pub fn apartment_id(building: usize, floor: usize, unit: usize) -> u64 {
    ((building as u64) << 32) | ((floor as u64) << 8) | unit as u64
}

pub fn layout_for_id(id: u64) -> LayoutKind {
    match id.wrapping_mul(0x9e37_79b9_7f4a_7c15) % 6 {
        0 => LayoutKind::Studio,
        1 => LayoutKind::OneBedroom,
        2 => LayoutKind::TwoBedroom,
        3 => LayoutKind::ThreeBedroom,
        4 => LayoutKind::LongHall,
        _ => LayoutKind::Corner,
    }
}

/// Apartment instances occupy a wide X/Z grid well outside the playable city.
/// Rows advance in Z; no instance is stacked above another.
pub fn instance_origin(id: u64) -> Vec3 {
    let building = (id >> 32) as f32;
    let floor = ((id >> 8) & 0x00ff_ffff) as f32;
    let unit = (id & 0xff) as f32;
    Vec3::new(
        -20_000. + building * 256. + unit * 48.,
        0.,
        20_000. + floor * 48.,
    )
}

pub fn instance_size(id: u64) -> [f32; 2] {
    match layout_for_id(id) {
        LayoutKind::Studio => [14., 12.],
        LayoutKind::OneBedroom => [18., 14.],
        LayoutKind::TwoBedroom => [20., 16.],
        LayoutKind::ThreeBedroom => [22., 18.],
        LayoutKind::LongHall => [16., 22.],
        LayoutKind::Corner => [20., 18.],
    }
}

pub fn instance_spawn(id: u64) -> Vec3 {
    let origin = instance_origin(id);
    Vec3::new(origin.x, 1., origin.z + instance_size(id)[1] * 0.22)
}

pub fn instance_exit(id: u64) -> Vec3 {
    let origin = instance_origin(id);
    Vec3::new(origin.x, 1., origin.z - instance_size(id)[1] * 0.43)
}

/// Build one lightweight, self-contained apartment mesh for a single active zone.
pub fn instance_mesh(id: u64) -> Mesh {
    let origin = instance_origin(id);
    let x = origin.x;
    let z = origin.z;
    let [width, depth] = instance_size(id);
    let half_width = width * 0.5;
    let half_depth = depth * 0.5;
    let mut mesh = Mesh::default();
    mesh.cuboid(
        [x, -0.08, z],
        [width, 0.16, depth],
        [0.56, 0.53, 0.47],
        1,
        0.,
    );
    mesh.cuboid([x, 3.25, z], [width, 0.12, depth], [0.72, 0.7, 0.64], 1, 0.);
    mesh.cuboid(
        [x - half_width + 0.1, 1.58, z],
        [0.2, 3.2, depth],
        [0.76, 0.73, 0.68],
        1,
        0.,
    );
    mesh.cuboid(
        [x + half_width - 0.1, 1.58, z],
        [0.2, 3.2, depth],
        [0.76, 0.73, 0.68],
        1,
        0.,
    );
    mesh.cuboid(
        [x, 1.58, z + half_depth - 0.1],
        [width, 3.2, 0.2],
        [0.76, 0.73, 0.68],
        1,
        0.,
    );
    // Front wall has a fixed, walkable portal at the exact exit point.
    mesh.cuboid(
        [x - (half_width + 1.) * 0.5, 1.58, z - half_depth + 0.1],
        [half_width - 1., 3.2, 0.2],
        [0.76, 0.73, 0.68],
        1,
        0.,
    );
    mesh.cuboid(
        [x + (half_width + 1.) * 0.5, 1.58, z - half_depth + 0.1],
        [half_width - 1., 3.2, 0.2],
        [0.76, 0.73, 0.68],
        1,
        0.,
    );
    let kind = layout_for_id(id);
    match kind {
        LayoutKind::Studio => {}
        LayoutKind::OneBedroom => mesh.cuboid(
            [x - 2.2, 1.45, z + 1.5],
            [0.12, 2.9, depth * 0.54],
            [0.75, 0.72, 0.67],
            1,
            0.,
        ),
        LayoutKind::TwoBedroom => {
            mesh.cuboid(
                [x + 2., 1.45, z + 2.6],
                [0.12, 2.9, depth * 0.48],
                [0.75, 0.72, 0.67],
                1,
                0.,
            );
            mesh.cuboid(
                [x - 3., 1.45, z],
                [width * 0.35, 2.9, 0.12],
                [0.75, 0.72, 0.67],
                1,
                0.,
            );
        }
        LayoutKind::ThreeBedroom => {
            mesh.cuboid(
                [x, 1.45, z + 3.5],
                [0.12, 2.9, depth * 0.48],
                [0.75, 0.72, 0.67],
                1,
                0.,
            );
            mesh.cuboid(
                [x - 4., 1.45, z - 1.],
                [width * 0.34, 2.9, 0.12],
                [0.75, 0.72, 0.67],
                1,
                0.,
            );
            mesh.cuboid(
                [x + 3., 1.45, z - 5.],
                [width * 0.32, 2.9, 0.12],
                [0.75, 0.72, 0.67],
                1,
                0.,
            );
        }
        LayoutKind::LongHall => mesh.cuboid(
            [x + 4., 1.45, z + 0.5],
            [0.12, 2.9, depth * 0.74],
            [0.75, 0.72, 0.67],
            1,
            0.,
        ),
        LayoutKind::Corner => {
            mesh.cuboid(
                [x - 3., 1.45, z + 3.],
                [0.12, 2.9, depth * 0.44],
                [0.75, 0.72, 0.67],
                1,
                0.,
            );
            mesh.cuboid(
                [x - 1., 1.45, z - 4.],
                [width * 0.34, 2.9, 0.12],
                [0.75, 0.72, 0.67],
                1,
                0.,
            );
        }
    }
    mesh.cuboid(
        [x + 5.8, 0.42, z + 4.6],
        [2.8, 0.84, 2.2],
        [0.45, 0.31, 0.2],
        10,
        0.,
    );
    mesh.cuboid(
        [x - 5.8, 0.46, z + 4.8],
        [2.5, 0.92, 1.7],
        [0.56, 0.58, 0.56],
        1,
        0.,
    );
    mesh.cuboid(
        [x + 5.8, 0.4, z - 3.8],
        [3.1, 0.8, 1.8],
        [0.38, 0.42, 0.45],
        4,
        0.,
    );
    mesh
}

/// Collision boundary for the active instance, including its invisible perimeter.
pub fn instance_blocked(id: u64, p: Vec3, radius: f32) -> bool {
    let o = instance_origin(id);
    if p.y < 0.5 || p.y > 2.8 {
        return false;
    }
    let x = p.x - o.x;
    let z = p.z - o.z;
    let [width, depth] = instance_size(id);
    let half_width = width * 0.5;
    let half_depth = depth * 0.5;
    let side_wall = x.abs() + radius > half_width - 0.2 && z.abs() < half_depth + radius;
    let back_wall = z + radius > half_depth - 0.2 && x.abs() < half_width + radius;
    let front_wall =
        z - radius < -half_depth + 0.2 && z + radius > -half_depth - 0.2 && x.abs() > 1.1 + radius;
    let exterior_boundary = x.abs() > half_width + 0.2 || z < -half_depth - 0.5;
    side_wall || back_wall || front_wall || exterior_boundary
}
pub(super) struct Fixture {
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub color: [f32; 3],
    pub material: u8,
}
pub(super) fn room_variant(b: &Building, floor: usize, side: f32) -> usize {
    let side_id = usize::from(side > 0.0);
    // The same stable address selects both the visual and collision layout.
    match layout_for_id(apartment_id(b.id, floor, side_id)) {
        LayoutKind::Studio => 0,
        LayoutKind::OneBedroom => 1,
        LayoutKind::TwoBedroom => 2,
        LayoutKind::ThreeBedroom => 3,
        LayoutKind::LongHall => 4,
        LayoutKind::Corner => 5,
    }
}

/// Add a short room divider with a two-metre doorway in each apartment.
pub(super) fn room_partitions(b: &Building, floor: usize, side: f32) -> Vec<Collider> {
    let layout = room_variant(b, floor, side);
    let inner = 3.55;
    let outer = b.size.x / 2.0 - 0.8;
    let door_center = if layout == 4 { 8.6 } else { 6.8 };
    let horizontal = if layout == 0 {
        Vec::new()
    } else {
        [(inner, door_center - 1.0), (door_center + 1.0, outer)]
            .into_iter()
            .filter_map(|(start, end)| {
                (end > start).then_some(Collider {
                    p: Vec3::new(
                        b.p.x + side * (start + end) * 0.5,
                        floor as f32 * 4.0 + 1.5,
                        b.p.z,
                    ),
                    size: Vec3::new(end - start, 3.0, 0.16),
                    movable: false,
                    breakable: false,
                })
            })
            .collect()
    };
    let mut partitions = horizontal;
    if matches!(layout, 2 | 3 | 5) {
        let center_x = b.p.x + side * (b.size.x / 2.0 - 7.0);
        let z = b.p.z + if layout == 5 { 3.8 } else { 0. };
        partitions.push(Collider {
            p: Vec3::new(center_x, floor as f32 * 4.0 + 1.5, z),
            size: Vec3::new(0.16, 3.0, 4.0),
            movable: false,
            breakable: false,
        });
    }
    partitions
}

pub(super) fn room_fixtures(b: &Building, floor: usize, side: f32) -> Vec<Fixture> {
    let y = floor as f32 * 4.0;
    let rx = b.p.x + side * (b.size.x / 2.0 - 7.0);
    let layout = room_variant(b, floor, side);
    let wood = match layout {
        0 => [0.62, 0.44, 0.28],
        1 => [0.72, 0.72, 0.66],
        2 => [0.35, 0.43, 0.49],
        3 => [0.48, 0.53, 0.38],
        4 => [0.59, 0.47, 0.34],
        _ => [0.43, 0.51, 0.58],
    };
    let mut out = Vec::new();
    let mut add = |position, size, color, material| {
        out.push(Fixture {
            position,
            size,
            color,
            material,
        })
    };
    match layout {
        0 => {
            // Family plan: a second compact bed and a bedside cabinet.
            add(
                [rx - side * 2.8, y + 0.66, b.p.z - 8.0],
                [2.3, 1.25, 3.4],
                wood,
                10,
            );
            add(
                [rx - side * 2.8, y + 1.32, b.p.z - 8.8],
                [2.2, 0.12, 1.7],
                [0.9, 0.88, 0.82],
                1,
            );
            add(
                [rx + side * 2.1, y + 0.4, b.p.z + 8.0],
                [1.7, 0.8, 1.5],
                [0.48, 0.59, 0.64],
                1,
            );
        }
        1 => {
            // Work-from-home plan: desk, chair and a wall cabinet.
            add(
                [rx - side * 2.2, y + 0.72, b.p.z + 8.0],
                [3.2, 0.16, 1.6],
                wood,
                10,
            );
            add(
                [rx - side * 2.2, y + 0.38, b.p.z + 6.8],
                [0.85, 0.72, 0.8],
                [0.19, 0.23, 0.27],
                4,
            );
            add(
                [rx - side * 2.2, y + 2.25, b.p.z + 11.0],
                [2.8, 1.15, 0.35],
                wood,
                10,
            );
        }
        2 => {
            // Dining plan: a larger table with four chairs.
            add([rx, y + 0.78, b.p.z + 9.0], [3.4, 0.16, 2.2], wood, 10);
            for dx in [-1.8, 1.8] {
                for dz in [-0.7, 0.7] {
                    add(
                        [rx + dx, y + 0.46, b.p.z + 9.0 + dz],
                        [0.65, 0.86, 0.62],
                        [0.38, 0.34, 0.28],
                        10,
                    );
                }
            }
        }
        _ => {
            // Open plan: bookcase, desk and a reading chair.
            add(
                [rx + side * 3.0, y + 1.15, b.p.z + 8.5],
                [1.5, 2.3, 2.8],
                wood,
                10,
            );
            add(
                [rx - side * 2.0, y + 0.72, b.p.z + 8.0],
                [2.7, 0.16, 1.5],
                [0.35, 0.4, 0.44],
                4,
            );
            add(
                [rx - side * 2.0, y + 0.48, b.p.z + 6.5],
                [1.0, 0.95, 1.0],
                [0.55, 0.35, 0.25],
                10,
            );
        }
    }
    if matches!(layout, 2 | 3 | 5) {
        // Extra bedroom furniture for the larger and corner plans.
        add(
            [rx - side * 2.5, y + 0.66, b.p.z + 7.0],
            [2.2, 1.25, 3.2],
            wood,
            10,
        );
        add(
            [rx - side * 2.5, y + 1.3, b.p.z + 6.2],
            [2.1, 0.12, 1.6],
            [0.9, 0.88, 0.82],
            1,
        );
    }
    out
}
pub(super) fn fixtures(b: &Building, floor: usize) -> Vec<Fixture> {
    if floor == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let y = floor as f32 * 4.;
    let variant = (b.id + floor) % 4;
    let wood = match variant {
        0 => [0.57, 0.42, 0.29],
        1 => [0.73, 0.7, 0.62],
        2 => [0.32, 0.39, 0.42],
        _ => [0.45, 0.49, 0.35],
    };
    let mut add = |p, s, c, mat| {
        out.push(Fixture {
            position: p,
            size: s,
            color: c,
            material: mat,
        })
    };
    for side in [-1., 1.] {
        let x = b.p.x + side * (b.size.x / 2. - 1.2);
        let z = b.p.z;
        add([x, y + 0.5, z - 1.], [1.5, 1., 3.2], wood, 10);
        add([x, y + 1.03, z - 1.], [1.6, 0.08, 3.3], [0.8, 0.8, 0.74], 1);
        add(
            [x, y + 1.085, z - 1.9],
            [1.05, 0.025, 0.95],
            [0.13, 0.17, 0.18],
            4,
        );
        add(
            [x, y + 1.09, z - 0.2],
            [0.85, 0.03, 0.8],
            [0.28, 0.31, 0.32],
            4,
        );
        add(
            [x + side * 0.25, y + 1.27, z - 0.2],
            [0.05, 0.35, 0.05],
            [0.7, 0.74, 0.76],
            4,
        );
        add([x, y + 2.3, z - 1.], [0.6, 0.8, 3.1], wood, 10);
        add([x, y + 1.1, z + 2.], [1.3, 2.2, 1.1], [0.75, 0.78, 0.77], 4);
        add(
            [x - side * 0.67, y + 1.25, z + 2.],
            [0.035, 0.7, 0.06],
            [0.17, 0.2, 0.21],
            4,
        );
        // Bathroom occupies the rear outer corner, clear of the existing bed/sofa/wardrobe targets.
        let bx = b.p.x + side * (b.size.x / 2. - 2.4);
        let bz = b.p.z + b.size.z / 2. - 2.;
        add([bx, y + 0.08, bz], [3.4, 0.08, 3.1], [0.64, 0.7, 0.68], 1);
        add(
            [bx + side * 0.6, y + 0.32, bz + 0.35],
            [0.65, 0.64, 0.9],
            [0.86, 0.87, 0.81],
            1,
        );
        add(
            [bx + side * 0.6, y + 0.8, bz + 0.65],
            [0.65, 0.5, 0.18],
            [0.86, 0.87, 0.81],
            1,
        );
        add(
            [bx - side * 0.7, y + 0.8, bz + 0.3],
            [0.75, 0.15, 0.6],
            [0.82, 0.85, 0.83],
            1,
        );
        add(
            [bx - side * 0.7, y + 1.6, bz + 0.65],
            [0.7, 0.8, 0.04],
            [0.42, 0.52, 0.56],
            3,
        );
        add(
            [bx, y + 0.18, bz - 0.8],
            [1.2, 0.12, 0.85],
            [0.8, 0.83, 0.8],
            1,
        );
        // Thin shower screen, away from the central hall and lift.
        add(
            [bx - side * 0.85, y + 1.2, bz - 0.85],
            [0.06, 2.2, 1.05],
            [0.49, 0.6, 0.62],
            3,
        );
    }
    out
}
/// Rental homes use one kitchen/bathroom corner at street level, not a tower lobby.
pub(super) fn ground_fixtures(b: &Building) -> Vec<Fixture> {
    fixtures(b, 1)
        .into_iter()
        .filter(|f| f.position[0] > b.p.x)
        .map(|mut f| {
            f.position[1] = (f.position[1] - 4.).max(f.size[1] / 2.);
            f
        })
        .collect()
}
pub(super) fn ground_colliders(b: &Building) -> Vec<Collider> {
    ground_fixtures(b)
        .into_iter()
        .map(|f| Collider {
            p: Vec3::new(f.position[0], f.position[1], f.position[2]),
            size: Vec3::new(f.size[0], f.size[1], f.size[2]),
            movable: false,
            breakable: false,
        })
        .collect()
}
pub(super) fn colliders(b: &Building, floor: usize) -> Vec<Collider> {
    fixtures(b, floor)
        .into_iter()
        .map(|f| Collider {
            p: Vec3::new(f.position[0], f.position[1], f.position[2]),
            size: Vec3::new(f.size[0], f.size[1], f.size[2]),
            movable: false,
            breakable: false,
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn apartment_addresses_and_layouts_are_stable_and_unique() {
        let ids: Vec<_> = (0..12)
            .map(|floor| apartment_id(41, floor, floor % 4))
            .collect();
        let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len());
        let layouts: Vec<_> = (0..24)
            .map(|id| layout_for_id(apartment_id(9, id / 4, id % 4)))
            .collect();
        assert!(layouts.contains(&LayoutKind::Studio));
        assert!(layouts.contains(&LayoutKind::LongHall));
        assert_eq!(layouts[3], layout_for_id(apartment_id(9, 0, 3)));
    }
    #[test]
    fn apartment_instances_are_flat_spread_and_have_a_walkable_portal() {
        let id = apartment_id(12, 7, 2);
        let other = apartment_id(12, 8, 2);
        let origin = instance_origin(id);
        let front = origin.z - instance_size(id)[1] * 0.5 + 0.1;
        let other_origin = instance_origin(other);
        assert_eq!(origin.y, other_origin.y);
        assert!((origin.z - other_origin.z).abs() >= 48.);
        assert!(instance_mesh(id).vertices.len() > 100);
        assert!(!instance_blocked(
            id,
            Vec3::new(origin.x, 1., origin.z),
            0.45
        ));
        assert!(!instance_blocked(id, Vec3::new(origin.x, 1., front), 0.45));
        assert!(instance_blocked(
            id,
            Vec3::new(origin.x + 5., 1., front),
            0.45
        ));
    }
    #[test]
    fn rental_fixtures_fit_ground_floor_without_covering_front_door() {
        let b = Building {
            id: 9,
            name: "Apartments",
            p: Vec3::new(-220., 0., 210.),
            size: Vec3::new(20., 5., 22.),
            style: super::super::Style::House,
            interior: true,
        };
        assert!(!ground_fixtures(&b).is_empty());
        for c in ground_colliders(&b) {
            assert!(c.p.y - c.size.y / 2. >= 0.);
            assert!(c.p.y + c.size.y / 2. < b.size.y);
            assert!(!c.contains(b.entrance(), 0.45));
            assert!((c.p.x - b.p.x).abs() + c.size.x / 2. < b.size.x / 2.);
            assert!((c.p.z - b.p.z).abs() + c.size.z / 2. < b.size.z / 2.);
        }
    }
    #[test]
    fn fixtures_fit_apartments_and_never_obstruct_lift() {
        let b = Building {
            id: 9,
            name: "Office",
            p: Vec3::new(110., 0., 100.),
            size: Vec3::new(28., 64., 28.),
            style: super::super::Style::Glass,
            interior: true,
        };
        assert!(fixtures(&b, 0).is_empty());
        for floor in [1, 4, 12] {
            for f in fixtures(&b, floor) {
                assert!((f.position[0] - b.p.x).abs() + f.size[0] / 2. < b.size.x / 2.);
                assert!((f.position[2] - b.p.z).abs() + f.size[2] / 2. < b.size.z / 2.);
                assert!(f.position[1] - f.size[1] / 2. >= floor as f32 * 4.);
                assert!(f.position[1] + f.size[1] / 2. < (floor + 1) as f32 * 4.);
                assert!((f.position[0] - b.p.x).abs() - f.size[0] / 2. > 3.);
            }
        }
        assert_eq!(fixtures(&b, 1).len(), colliders(&b, 1).len());
    }
}
