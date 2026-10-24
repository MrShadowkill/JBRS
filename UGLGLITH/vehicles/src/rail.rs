//! Freight equipment inspired by diesel-electric hood locomotives and steel boxcars.
use engine_core::Mesh;
const STEEL: [f32; 3] = [0.2, 0.24, 0.27];
const BLACK: [f32; 3] = [0.055, 0.065, 0.075];
const RAIL: [f32; 3] = [0.65, 0.69, 0.7];
pub const LOCOMOTIVE_NAME: &str = "Ironvale D90";
pub const WAGON_NAME: &str = "Ironvale Cargo 25";

pub fn locomotive(detail: bool) -> Mesh {
    let mut m = Mesh::default();
    m.cuboid([0., 0., 0.], [3.8, 0.7, 9.], STEEL, 1, 0.);
    m.cuboid(
        [0., 1.55, -0.65],
        [2.55, 2.5, 6.6],
        [0.72, 0.27, 0.08],
        1,
        0.,
    );
    m.cuboid([0., 2., 2.6], [3.25, 3., 2.3], [0.8, 0.35, 0.12], 1, 0.);
    m.cuboid([0., 3.57, 2.6], [3.5, 0.18, 2.6], STEEL, 1, 0.);
    for side in [-1., 1.] {
        m.cuboid(
            [side * 0.82, 2.65, 3.77],
            [1.25, 0.8, 0.035],
            [0.12, 0.24, 0.3],
            1,
            0.,
        );
        m.cuboid(
            [side * 1.64, 2.6, 2.65],
            [0.035, 0.85, 1.3],
            [0.12, 0.24, 0.3],
            1,
            0.,
        );
        for z in [-2.65, 2.65] {
            m.cuboid([side * 1.36, -0.48, z], [0.35, 0.5, 2.], BLACK, 1, 0.);
        }
        if detail {
            for z in [-3.9, -2.7, -1.5, -0.3, 0.9] {
                m.cuboid([side * 1.78, 0.9, z], [0.045, 1.1, 0.045], RAIL, 1, 0.);
            }
            for y in [1.15, 1.43] {
                m.cuboid([side * 1.78, y, -1.5], [0.045, 0.045, 5.9], RAIL, 1, 0.);
            }
            for z in [-3.4, -2.9, -2.4, -1.9, -1.4, -0.9] {
                m.cuboid([side * 1.29, 1.9, z], [0.03, 1.2, 0.035], BLACK, 1, 0.);
            }
            for y in [0.1, 0.38, 0.66] {
                m.cuboid([side * 1.7, y, 3.65], [0.5, 0.06, 0.22], STEEL, 1, 0.);
            }
            for z in [-2., -0.5] {
                m.cylinder([side * 0.62, 2.81, z], 0.48, 0.08, BLACK, 1, 12);
            }
        }
    }
    for z in [-4.6, 4.6] {
        m.cuboid([0., -0.25, z], [0.38, 0.3, 0.6], STEEL, 1, 0.);
        m.cuboid([0., 0.2, z * 0.97], [3.5, 0.7, 0.15], BLACK, 1, 0.);
    }
    m.cuboid([0., 3.1, 3.79], [0.6, 0.22, 0.03], [0.98, 0.9, 0.6], 1, 0.);
    if detail {
        m.cylinder([0., 2.85, -0.6], 0.17, 0.45, BLACK, 1, 8);
        m.cuboid([0., 3.15, 1.5], [0.7, 0.12, 0.3], RAIL, 1, 0.);
    }
    if detail {
        m.vertices.extend(running_gear().vertices);
    }
    m
}

/// Additive details leave the existing openable gold wagon and robbery state intact.
pub fn wagon_equipment() -> Mesh {
    let mut m = Mesh::default();
    for side in [-1., 1.] {
        for i in -5..=5 {
            m.cuboid(
                [side * 1.62, 1.8, i as f32 * 0.65],
                [0.055, 2.2, 0.055],
                STEEL,
                1,
                0.,
            );
        }
        m.cuboid([side * 1.66, 1.8, 0.], [0.06, 1.95, 2.2], STEEL, 1, 0.);
        for z in [-1., 1.] {
            m.cuboid([side * 1.71, 1.8, z], [0.04, 1.85, 0.055], RAIL, 1, 0.);
        }
        m.cuboid([side * 1.74, 1.7, 0.25], [0.035, 0.5, 0.055], RAIL, 1, 0.);
        for y in [0.5, 0.85, 1.2, 1.55, 1.9, 2.25, 2.6] {
            m.cuboid([side * 1.64, y, -3.6], [0.18, 0.04, 0.42], RAIL, 1, 0.);
        }
    }
    for z in [-4.45, 4.45] {
        m.cuboid([0., -0.15, z], [0.35, 0.22, 0.6], STEEL, 1, 0.);
    }
    m
}

/// Two bogies, four axles, flanged steel wheels and visible suspension frames.
pub fn running_gear() -> Mesh {
    let mut m = Mesh::default();
    for z in [-2.65, 2.65] {
        for side in [-1., 1.] {
            m.cuboid([side * 1.45, -0.25, z], [0.2, 0.3, 2.], STEEL, 1, 0.);
            for dz in [-0.6, 0.6] {
                let mut wheel = Mesh::default();
                super::body::wheel(&mut wheel, [0.; 3], side, 16, false);
                for v in &mut wheel.vertices {
                    v.position[0] += side * 1.48;
                    v.position[1] = v.position[1] * 1.4 - 0.4;
                    v.position[2] = v.position[2] * 1.4 + z + dz;
                    v.color[0] = v.color[0] * 0.65 + 0.08;
                    v.color[1] = v.color[1] * 0.65 + 0.08;
                    v.color[2] = v.color[2] * 0.65 + 0.08;
                }
                m.vertices.extend(wheel.vertices);
            }
            for dz in [-0.28, 0.28] {
                m.cuboid(
                    [side * 1.58, -0.22, z + dz],
                    [0.13, 0.3, 0.18],
                    BLACK,
                    1,
                    0.,
                );
            }
        }
        for dz in [-0.6, 0.6] {
            m.cuboid([0., -0.4, z + dz], [3., 0.14, 0.14], STEEL, 1, 0.);
        }
    }
    m
}

#[cfg(test)]
mod tests {
    #[test]
    fn rail_geometry_stays_inside_streaming_bounds() {
        for mesh in [
            super::locomotive(false),
            super::locomotive(true),
            super::wagon_equipment(),
            super::running_gear(),
        ] {
            assert!(mesh.vertices.len() / 3 < 3000);
            for v in &mesh.vertices {
                assert!(v.position.iter().all(|x| x.is_finite()));
                assert!(
                    v.position[0].abs() < 2.1
                        && v.position[1] > -1.
                        && v.position[1] < 4.
                        && v.position[2].abs() < 5.1
                );
            }
        }
        assert!(super::locomotive(false).vertices.len() < super::locomotive(true).vertices.len());
    }
}
