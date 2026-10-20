//! Deterministic architecture: dense neutral city blocks with distinct crowns and facades.
use super::{Building, Collider, Mesh, Vec3};

#[derive(Clone, Copy)]
struct Palette {
    wall: [f32; 3],
    trim: [f32; 3],
    glass: [f32; 3],
}
const PALETTES: [Palette; 7] = [
    Palette {
        wall: [0.2, 0.25, 0.28],
        trim: [0.42, 0.46, 0.47],
        glass: [0.16, 0.23, 0.27],
    },
    Palette {
        wall: [0.65, 0.63, 0.56],
        trim: [0.38, 0.39, 0.37],
        glass: [0.23, 0.29, 0.31],
    },
    Palette {
        wall: [0.38, 0.43, 0.45],
        trim: [0.72, 0.73, 0.68],
        glass: [0.14, 0.24, 0.3],
    },
    Palette {
        wall: [0.49, 0.3, 0.23],
        trim: [0.7, 0.65, 0.54],
        glass: [0.19, 0.24, 0.26],
    },
    Palette {
        wall: [0.71, 0.67, 0.56],
        trim: [0.47, 0.43, 0.36],
        glass: [0.22, 0.28, 0.29],
    },
    Palette {
        wall: [0.25, 0.27, 0.28],
        trim: [0.53, 0.53, 0.5],
        glass: [0.18, 0.23, 0.25],
    },
    Palette {
        wall: [0.31, 0.4, 0.4],
        trim: [0.58, 0.64, 0.61],
        glass: [0.17, 0.29, 0.32],
    },
];
impl Building {
    pub fn city_architecture(&self) -> bool {
        matches!(
            self.name,
            "Office" | "Westhaven" | "Apartments" | "Residence" | "City Annex"
        )
    }
    pub fn architecture_variant(&self) -> usize {
        ((self.p.x as i32).unsigned_abs() as usize / 10
            + (self.p.z as i32).unsigned_abs() as usize / 10 * 3
            + self.id)
            % PALETTES.len()
    }
    /// Roof equipment stays inside the footprint and shares exact mesh/collider dimensions.
    pub(super) fn crown(&self) -> Vec<Collider> {
        if !self.city_architecture() {
            return Vec::new();
        }
        let h = self.size.y;
        let w = self.size.x;
        let d = self.size.z;
        let tall = h >= 24.;
        let variant = self.architecture_variant();
        let mut boxes = Vec::new();
        let mut add = |x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32| {
            boxes.push(Collider {
                p: Vec3::new(self.p.x + x, y, self.p.z + z),
                size: Vec3::new(sx, sy, sz),
                movable: false,
                breakable: false,
            })
        };
        if tall {
            match variant {
                0 | 4 => {
                    add(0., h + 2., 0., w * 0.76, 4., d * 0.76);
                    add(0., h + 6., 0., w * 0.48, 4., d * 0.48);
                    add(0., h + 9.5, 0., w * 0.24, 3., d * 0.24);
                }
                1 => {
                    add(-w * 0.18, h + 4., 0., w * 0.36, 8., d * 0.55);
                    add(w * 0.18, h + 2., 0., w * 0.32, 4., d * 0.55);
                }
                2 | 6 => {
                    add(0., h + 2., 0., w * 0.55, 4., d * 0.55);
                    add(w * 0.15, h + 8., 0., 1., 12., 1.);
                }
                _ => {
                    add(0., h + 1.5, d * 0.12, w * 0.42, 3., d * 0.4);
                    add(-w * 0.24, h + 1., -d * 0.2, w * 0.18, 2., d * 0.2);
                }
            }
        } else {
            add(w * 0.15, h + 0.8, d * 0.15, w * 0.32, 1.6, d * 0.25);
        }
        boxes
    }
    pub(super) fn city_mesh(&self, m: &mut Mesh, lod: u8) {
        let palette = PALETTES[self.architecture_variant()];
        let variant = self.architecture_variant();
        let w = self.size.x;
        let d = self.size.z;
        let h = self.size.y;
        let x = self.p.x;
        let z = self.p.z;
        if lod == 2 {
            let start = m.vertices.len();
            m.cuboid([x, h / 2., z], [w, h, d], palette.wall, 0, 0.);
            // Nine flat square windows are enough to read as a block in Potato mode.
            for row in 0..3 {
                for column in 0..3 {
                    let wx = x - w * 0.34 + column as f32 * w * 0.34;
                    let wy = (h * 0.22 + row as f32 * h * 0.24).min(h - 1.2);
                    m.cuboid(
                        [wx, wy, z - d / 2. - 0.035],
                        [1.15, 1.05, 0.07],
                        if (row + column + self.id).is_multiple_of(4) {
                            [0.72, 0.61, 0.36]
                        } else {
                            palette.glass
                        },
                        0,
                        0.,
                    );
                }
            }
            let door_z = z - d / 2. - 0.06;
            // A simple frame and slab; Potato mode has no animated facade doors.
            m.cuboid([x, 1.4, door_z], [2.2, 2.8, 0.12], palette.trim, 0, 0.);
            m.cuboid(
                [x, 1.35, door_z - 0.08],
                [1.45, 2.5, 0.1],
                [0.28, 0.2, 0.14],
                0,
                0.,
            );
            for crown in self.crown() {
                m.cuboid(
                    [crown.p.x, crown.p.y, crown.p.z],
                    [crown.size.x, crown.size.y, crown.size.z],
                    palette.wall,
                    0,
                    0.,
                );
            }
            for vertex in &mut m.vertices[start..] {
                // A low alpha tag bypasses atlas sampling without triggering grass sway.
                vertex.color[3] = -0.6;
            }
            return;
        }
        for wall in self.shell_colliders() {
            m.cuboid(
                [wall.p.x, wall.p.y, wall.p.z],
                [wall.size.x, wall.size.y, wall.size.z],
                palette.wall,
                1,
                0.,
            );
        }
        m.cuboid(
            [x, -0.12, z],
            [w + 0.8, 0.24, d + 0.8],
            [0.42, 0.43, 0.41],
            1,
            0.,
        );
        // External mullions, cornices and window panels are planar and batched per chunk.
        let floors = (h / 4.).ceil() as usize;
        let stride = match lod {
            0 => 1,
            1 => 2,
            _ => 4,
        };
        let columns = match lod {
            0 => 6,
            1 => 4,
            _ => 2,
        };
        for face in 0..4 {
            let span = if face < 2 { w } else { d };
            let point = |u: f32, y: f32, offset: f32| match face {
                0 => [x + u, y, z - d / 2. - offset],
                1 => [x - u, y, z + d / 2. + offset],
                2 => [x - w / 2. - offset, y, z - u],
                _ => [x + w / 2. + offset, y, z + u],
            };
            for floor in (0..floors).step_by(stride) {
                let y = floor as f32 * 4.;
                let top = (y + stride as f32 * 4. - 0.85).min(h - 0.45);
                for column in 0..columns {
                    let a = -span / 2. + 0.9 + column as f32 * (span - 1.8) / columns as f32;
                    let b = -span / 2. + 0.9 + (column + 1) as f32 * (span - 1.8) / columns as f32
                        - 0.4;
                    // Never cover the real 3m entrance with a decorative facade panel.
                    if face == 0 && floor == 0 && a < 1.7 && b > -1.7 {
                        continue;
                    }
                    let warm =
                        (self.id * 17 + floor * 11 + column * 3 + face * 5).is_multiple_of(13);
                    let color = if warm {
                        [0.63, 0.56, 0.37]
                    } else {
                        palette.glass
                    };
                    m.quad(
                        [
                            point(a, y + 0.8, 0.365),
                            point(b, y + 0.8, 0.365),
                            point(b, top, 0.365),
                            point(a, top, 0.365),
                        ],
                        color,
                        3,
                        if warm { 0.48 } else { 0. },
                    );
                    if lod == 0 {
                        let split = a + (b - a) * 0.35;
                        m.quad(
                            [
                                point(a, top - 0.18, 0.37),
                                point(split, top - 0.18, 0.37),
                                point(split, y + 0.95, 0.37),
                                point(a, y + 0.95, 0.37),
                            ],
                            color.map(|v| (v * 1.2).min(1.)),
                            3,
                            0.,
                        );
                    }
                }
                if floor > 0 && (variant == 1 || variant == 3 || variant == 4 || floor % 3 == 0) {
                    m.quad(
                        [
                            point(-span / 2., y - 0.13, 0.38),
                            point(span / 2., y - 0.13, 0.38),
                            point(span / 2., y + 0.13, 0.38),
                            point(-span / 2., y + 0.13, 0.38),
                        ],
                        palette.trim,
                        1,
                        0.,
                    );
                }
            }
            if variant == 0 || variant == 2 || variant == 5 {
                for column in 1..columns {
                    let u = -span / 2. + column as f32 * span / columns as f32;
                    m.quad(
                        [
                            point(u - 0.13, 3.3, 0.39),
                            point(u + 0.13, 3.3, 0.39),
                            point(u + 0.13, h, 0.39),
                            point(u - 0.13, h, 0.39),
                        ],
                        palette.trim,
                        4,
                        0.,
                    );
                }
            }
            // Ground-floor stone plinth, with the entrance gap kept open.
            for (a, b) in if face == 0 {
                vec![(-span / 2., -1.65), (1.65, span / 2.)]
            } else {
                vec![(-span / 2., span / 2.)]
            } {
                m.quad(
                    [
                        point(a, 0., 0.38),
                        point(b, 0., 0.38),
                        point(b, 0.65, 0.38),
                        point(a, 0.65, 0.38),
                    ],
                    palette.trim,
                    1,
                    0.,
                );
            }
        }
        // Canopy and side columns frame a walkable, visible street entrance.
        if self.interior {
            m.cuboid(
                [x, 3.45, z - d / 2. - 0.7],
                [4.6, 0.18, 1.8],
                palette.trim,
                4,
                0.,
            );
            for side in [-1., 1.] {
                m.cuboid(
                    [x + side * 1.8, 1.6, z - d / 2. - 0.4],
                    [0.22, 3.2, 0.18],
                    palette.trim,
                    1,
                    0.,
                );
            }
        }
        // Ledges and asymmetric rooftop masses define the skyline even at low LOD.
        for y in [h, 3.8] {
            for side in [-1., 1.] {
                m.cuboid(
                    [x + side * w / 2., y, z],
                    [0.5, 0.3, d + 0.5],
                    palette.trim,
                    1,
                    0.,
                );
                m.cuboid(
                    [x, y, z + side * d / 2.],
                    [w + 0.5, 0.3, 0.5],
                    palette.trim,
                    1,
                    0.,
                );
            }
        }
        for part in self.crown() {
            m.cuboid(
                [part.p.x, part.p.y, part.p.z],
                [part.size.x, part.size.y, part.size.z],
                palette.wall,
                1,
                0.,
            );
        }
        if lod == 0 && matches!(variant, 1 | 3 | 4) {
            // Recessed-looking balcony bays sit on the sides, never across the entrance.
            for floor in 1..floors {
                if floor % 2 == 0 {
                    continue;
                }
                for side in [-1., 1.] {
                    let bx = x + side * (w / 2. + 0.42);
                    let by = floor as f32 * 4.;
                    m.cuboid([bx, by, z], [0.12, 0.16, 5.], palette.trim, 1, 0.);
                    m.cuboid(
                        [bx + side * 0.03, by + 0.7, z],
                        [0.06, 1.2, 5.],
                        [0.24, 0.27, 0.27],
                        4,
                        0.,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn building() -> Building {
        Building {
            id: 12,
            name: "Office",
            p: Vec3::new(110., 8., 100.),
            size: Vec3::new(28., 72., 28.),
            style: super::super::Style::Glass,
            interior: true,
        }
    }
    #[test]
    fn city_lod_keeps_roof_and_reduces_facade_geometry() {
        let b = building();
        let mut near = Mesh::default();
        let mut far = Mesh::default();
        b.mesh(&mut near, 0);
        b.mesh(&mut far, 2);
        assert!(near.vertices.len() < 18000);
        assert!(far.vertices.len() * 2 < near.vertices.len());
        assert!(far
            .vertices
            .iter()
            .all(|v| (v.color[3] + 0.6).abs() < 0.001));
        let top = |m: &Mesh| m.vertices.iter().map(|v| v.position[1]).fold(0., f32::max);
        assert_eq!(top(&near), top(&far));
        assert!(top(&near) > b.p.y + b.size.y);
        assert!(near
            .vertices
            .iter()
            .all(|v| v.position.iter().all(|p| p.is_finite())));
    }
    #[test]
    fn decorative_roof_matches_collision_and_stays_in_footprint() {
        let b = building();
        for c in b.crown() {
            assert!((c.p.x - b.p.x).abs() + c.size.x / 2. <= b.size.x / 2.);
            assert!((c.p.z - b.p.z).abs() + c.size.z / 2. <= b.size.z / 2.);
            let p = Vec3::new(c.p.x, c.p.y + b.p.y, c.p.z);
            assert!(b
                .colliders()
                .iter()
                .any(|collider| collider.contains(p, 0.)));
        }
    }
}
