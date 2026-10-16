//! Shared visible/physical fixtures for kitchen and bathroom corners of tower apartments.
use super::{Building, Collider, Vec3};
pub(super) struct Fixture {
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub color: [f32; 3],
    pub material: u8,
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
