//! Original J-Store architecture and a shared route/hazard description.
use super::*;
pub const CENTER: [f32; 2] = [260., 190.];
pub const DISPLAY_X: [f32; 2] = [-6., 6.];
pub const ROOF: f32 = 12.;
#[derive(Clone, Copy)]
pub struct Ramp {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub width: f32,
}
pub const RAMPS: [Ramp; 3] = [
    Ramp {
        start: [-10., 0., 10.],
        end: [10., 4., 10.],
        width: 3.2,
    },
    Ramp {
        start: [-11., 4., -10.],
        end: [-11., 8., 10.],
        width: 3.2,
    },
    Ramp {
        start: [11., 8., 10.],
        end: [11., 12., -10.],
        width: 3.2,
    },
];
impl Ramp {
    pub fn height(self, p: Vec3) -> Option<f32> {
        let dx = self.end[0] - self.start[0];
        let dz = self.end[2] - self.start[2];
        let len2 = dx * dx + dz * dz;
        let t = ((p.x - self.start[0]) * dx + (p.z - self.start[2]) * dz) / len2;
        let cross = ((p.x - self.start[0]) * dz - (p.z - self.start[2]) * dx).abs() / len2.sqrt();
        if (-0.025..=1.025).contains(&t)
            && cross < self.width / 2.
            && p.y >= self.start[1] - 0.3
            && p.y <= self.end[1] + 2.5
        {
            Some(self.start[1] + t.clamp(0., 1.) * (self.end[1] - self.start[1]))
        } else {
            None
        }
    }
}
pub fn ramp_height(b: &Building, p: Vec3) -> Option<f32> {
    let local = Vec3::new(p.x - b.p.x, p.y - b.p.y, p.z - b.p.z);
    RAMPS
        .into_iter()
        .filter_map(|r| r.height(local))
        .map(|h| h + b.p.y)
        .max_by(f32::total_cmp)
}
#[derive(Clone, Copy)]
pub struct Beam {
    pub p: [f32; 3],
    pub size: [f32; 3],
}
impl Beam {
    pub fn touches(self, p: Vec3, stance: u8) -> bool {
        let feet = p.y - 1.;
        let top = feet
            + if stance == 2 {
                0.65
            } else if stance == 1 {
                1.6
            } else {
                2.5
            };
        (p.x - self.p[0]).abs() < self.size[0] / 2. + 0.3
            && (p.z - self.p[2]).abs() < self.size[2] / 2. + 0.3
            && top > self.p[1] - self.size[1] / 2.
            && feet < self.p[1] + self.size[1] / 2.
    }
}
/// Local coordinates used unchanged by drawing and damage checks. Outer ramp lanes stay clear.
pub fn beams(time: f32) -> [Beam; 10] {
    let mut out = [Beam {
        p: [0.; 3],
        size: [0.; 3],
    }; 10];
    for (i, z) in [-6., 0., 6.].into_iter().enumerate() {
        out[i] = Beam {
            p: [0., 0.48, z],
            size: [18., 0.08, 0.08],
        };
    }
    for (i, z) in [-5., 1., 7.].into_iter().enumerate() {
        out[i + 3] = Beam {
            p: [0., 8.48, z],
            size: [18., 0.08, 0.08],
        };
    }
    out[6] = Beam {
        p: [0., 9.65, -2.],
        size: [18., 0.08, 0.08],
    };
    out[7] = Beam {
        p: [time.sin() * 6., 9.6, 4.],
        size: [0.08, 3.2, 0.08],
    };
    for (i, x) in [-6., 6.].into_iter().enumerate() {
        out[i + 8] = Beam {
            p: [x, 5.7, -5.],
            size: [0.08, 3.1, 0.08],
        };
    }
    out
}
#[derive(Clone, Copy)]
pub(super) struct Part {
    p: [f32; 3],
    size: [f32; 3],
    color: [f32; 3],
    material: u8,
}
fn parts() -> &'static [Part] {
    static PARTS: std::sync::OnceLock<Vec<Part>> = std::sync::OnceLock::new();
    PARTS.get_or_init(|| {
        let mut v = Vec::new();
        let mut add = |p, size, color, material| {
            v.push(Part {
                p,
                size,
                color,
                material,
            })
        };
        for x in DISPLAY_X {
            add([x, 0.65, 0.], [3., 1.3, 5.], [0.055, 0.09, 0.14], 4);
        }
        for x in [-6., 6.] {
            add([x, 4.55, 0.], [3.2, 1.1, 3.], [0.065, 0.1, 0.16], 4);
        }
        add([0., 4.5, 3.], [5., 1., 2.], [0.07, 0.12, 0.17], 4);
        // Pillars leave the central entrance, rear lift and side ramp lanes open.
        for x in [-8., 8.] {
            for z in [-5., 5.] {
                add([x, 6., z], [0.65, 12., 0.65], [0.12, 0.17, 0.23], 1);
            }
        }
        for x in [-6., 0., 6.] {
            for z in [-4., 3.] {
                add([x, 8.4, z], [2.3, 0.8, 2.3], [0.1, 0.17, 0.23], 4);
            }
        }
        // Compact service buildings on the roof; clear landings around the right stair opening.
        add([-6., 13.3, 4.], [7., 2.6, 5.], [0.64, 0.68, 0.7], 1);
        add([1., 13.3, 5.], [4., 2.6, 4.], [0.64, 0.68, 0.7], 1);
        v
    })
}

pub fn platform_height(b: &Building, p: Vec3) -> Option<f32> {
    let local = Vec3::new(p.x - b.p.x, p.y - b.p.y, p.z - b.p.z);
    parts()
        .iter()
        .filter(|part| {
            part.size[1] <= 1.5
                && (part.p[1] / 4.).floor() == ((local.y - 0.6) / 4.).floor().max(0.)
        })
        .filter_map(|part| {
            let top = part.p[1] + part.size[1] / 2.;
            if local.y >= top + 0.6
                && (local.x - part.p[0]).abs() < part.size[0] / 2.
                && (local.z - part.p[2]).abs() < part.size[2] / 2.
            {
                Some(b.p.y + top)
            } else {
                None
            }
        })
        .max_by(f32::total_cmp)
}

pub(super) fn colliders(b: &Building) -> Vec<Collider> {
    parts()
        .iter()
        .map(|p| Collider {
            p: Vec3::new(b.p.x + p.p[0], p.p[1], b.p.z + p.p[2]),
            size: Vec3::new(p.size[0], p.size[1], p.size[2]),
            movable: false,
            breakable: false,
        })
        .collect()
}
/// Four rectangles around each stair aperture; no hidden solid floor over the ramp.
fn slab(b: &Building, m: &mut Mesh, y: f32) {
    let (x0, x1, z0, z1) = if y == 4. {
        (-11.7, 11.7, 8.3, 11.7)
    } else if y == 8. {
        (-12.7, -9.3, -11., 11.)
    } else {
        (9.3, 12.7, -11., 11.)
    };
    for (a, c, d, e) in [
        (-14., x0, -14., 14.),
        (x1, 14., -14., 14.),
        (x0, x1, -14., z0),
        (x0, x1, z1, 14.),
    ] {
        m.cuboid(
            [b.p.x + (a + c) / 2., y - 0.06, b.p.z + (d + e) / 2.],
            [c - a, 0.12, e - d],
            [0.09, 0.2, 0.3],
            1,
            0.,
        );
    }
}
pub fn diamond(m: &mut Mesh, p: [f32; 3], r: f32) {
    for i in 0..6 {
        let a = i as f32 * std::f32::consts::TAU / 6.;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 6.;
        let at = |t: f32| [p[0] + r * t.cos(), p[1], p[2] + r * t.sin()];
        let c = if i % 2 == 0 {
            [0.3, 0.82, 0.95]
        } else {
            [0.65, 0.94, 1.]
        };
        m.triangle([[p[0], p[1] + r * 0.42, p[2]], at(a), at(b)], c, 9, 0.65);
        m.triangle(
            [at(a), [p[0], p[1] - r * 0.9, p[2]], at(b)],
            c.map(|v| v * 0.8),
            9,
            0.55,
        );
    }
}
impl Building {
    pub(super) fn jewelry_exterior(&self, m: &mut Mesh, lod: u8) {
        let x = self.p.x;
        let z = self.p.z;
        let frame = [0.055, 0.085, 0.12];
        m.cuboid([x, -0.06, z], [31., 0.12, 31.], [0.5, 0.54, 0.55], 1, 0.);
        m.cuboid(
            [x, 0.025, z - 27.],
            [5., 0.05, 23.],
            [0.5, 0.54, 0.55],
            1,
            0.,
        );
        for wall in self.shell_colliders() {
            m.cuboid(
                [wall.p.x, wall.p.y, wall.p.z],
                [wall.size.x, wall.size.y, wall.size.z],
                [0.14, 0.24, 0.31],
                3,
                0.,
            );
        }
        m.cuboid([x, 0.02, z], [28., 0.04, 28.], [0.1, 0.2, 0.29], 1, 0.);
        for side in [-1., 1.] {
            for offset in [-14., 0., 14.] {
                // Keep the real entrance in the center of the front facade free.
                if offset != 0. {
                    m.cuboid(
                        [x + offset, 6., z + side * 14.4],
                        [0.35, 12., 0.25],
                        frame,
                        4,
                        0.,
                    );
                }
                m.cuboid(
                    [x + side * 14.4, 6., z + offset],
                    [0.25, 12., 0.35],
                    frame,
                    4,
                    0.,
                );
            }
            for y in [0., 4., 8., 12.] {
                m.cuboid([x, y, z + side * 14.4], [28.7, 0.28, 0.25], frame, 4, 0.);
                m.cuboid([x + side * 14.4, y, z], [0.25, 0.28, 28.7], frame, 4, 0.);
            }
        }
        if lod == 0 {
            for side in [-1., 1.] {
                // Diamond plaques and paired door-side windows.
                diamond(m, [x + side * 10., 2.8, z - 14.5], 0.8);
                m.cuboid(
                    [x + side * 2., 1.6, z - 14.45],
                    [0.15, 3.2, 0.2],
                    frame,
                    4,
                    0.,
                );
            }
            for p in parts().iter().filter(|p| p.p[1] > 12.) {
                m.cuboid(
                    [x + p.p[0], p.p[1], z + p.p[2]],
                    p.size,
                    p.color,
                    p.material,
                    0.,
                );
                m.cuboid(
                    [x + p.p[0], 13., z + p.p[2] - p.size[2] / 2. - 0.02],
                    [1.1, 2., 0.05],
                    frame,
                    4,
                    0.,
                );
            }
            // Original J-STORE lettering, built as a small 5x7 block sign.
            let glyphs: [[u8; 7]; 7] = [
                [7, 2, 2, 2, 18, 18, 12],
                [0, 0, 0, 31, 0, 0, 0],
                [15, 16, 16, 14, 1, 1, 30],
                [31, 4, 4, 4, 4, 4, 4],
                [14, 17, 17, 17, 17, 17, 14],
                [30, 17, 17, 30, 20, 18, 17],
                [31, 16, 16, 30, 16, 16, 31],
            ];
            m.cuboid([x, 3.5, z - 14.6], [8., 0.9, 0.12], frame, 4, 0.);
            for (i, rows) in glyphs.iter().enumerate() {
                for (j, row) in rows.iter().enumerate() {
                    for bit in 0..5 {
                        if row & (1 << (4 - bit)) != 0 {
                            m.cuboid(
                                [
                                    x + 3.4 - (i * 6 + bit) as f32 * 0.16,
                                    3.95 - j as f32 * 0.12,
                                    z - 14.68,
                                ],
                                [0.12, 0.09, 0.025],
                                [0.5, 0.9, 1.],
                                9,
                                0.65,
                            );
                        }
                    }
                }
            }
        }
    }
    pub(super) fn jewelry_interior(&self, m: &mut Mesh) {
        let x = self.p.x;
        let z = self.p.z;
        for y in [4., 8.] {
            slab(self, m, y);
        }
        for p in parts().iter().filter(|p| p.p[1] <= 12.) {
            m.cuboid(
                [x + p.p[0], p.p[1], z + p.p[2]],
                p.size,
                p.color,
                p.material,
                0.,
            );
        }
        for ramp in RAMPS {
            let dx = ramp.end[0] - ramp.start[0];
            let dz = ramp.end[2] - ramp.start[2];
            let len = (dx * dx + dz * dz).sqrt();
            let nx = -dz / len * ramp.width / 2.;
            let nz = dx / len * ramp.width / 2.;
            let point = |p: [f32; 3], s: f32| [x + p[0] + nx * s, p[1], z + p[2] + nz * s];
            m.quad(
                [
                    point(ramp.start, -1.),
                    point(ramp.start, 1.),
                    point(ramp.end, 1.),
                    point(ramp.end, -1.),
                ],
                [0.2, 0.34, 0.46],
                4,
                0.,
            );
            for i in 1..20 {
                let t = i as f32 / 20.;
                let p = [
                    ramp.start[0] + dx * t,
                    ramp.start[1] + 4. * t + 0.008,
                    ramp.start[2] + dz * t,
                ];
                let a = point(p, -1.);
                let b = point(p, 1.);
                m.quad(
                    [
                        a,
                        b,
                        [b[0] + dx / len * 0.035, b[1], b[2] + dz / len * 0.035],
                        [a[0] + dx / len * 0.035, a[1], a[2] + dz / len * 0.035],
                    ],
                    [0.46, 0.58, 0.65],
                    1,
                    0.,
                );
            }
        }
        for floor in 0..3 {
            let y = floor as f32 * 4.;
            // Pale luminous ceiling panels and original diamond-pattern floor inlays.
            for xx in [-5., 5.] {
                m.cuboid(
                    [x + xx, y + 3.8, z],
                    [7., 0.06, 12.],
                    [0.65, 0.85, 1.],
                    9,
                    0.85,
                );
            }
            for xx in [-9., -3., 3., 9.] {
                for zz in [-8., -2., 4.] {
                    let p = [x + xx, y + 0.015, z + zz];
                    let r = 1.3;
                    let c = [0.16, 0.38, 0.57];
                    for (a, b) in [
                        ([-r, 0.], [0., r]),
                        ([0., r], [r, 0.]),
                        ([r, 0.], [0., -r]),
                        ([0., -r], [-r, 0.]),
                    ] {
                        m.quad(
                            [
                                [p[0] + a[0], p[1], p[2] + a[1]],
                                [p[0] + b[0], p[1], p[2] + b[1]],
                                [p[0] + b[0] + 0.035, p[1], p[2] + b[1]],
                                [p[0] + a[0] + 0.035, p[1], p[2] + a[1]],
                            ],
                            c,
                            9,
                            0.25,
                        );
                    }
                }
            }
        }
        for xx in DISPLAY_X {
            for zz in [-1.5, 0., 1.5] {
                diamond(m, [x + xx, 1.7, z + zz], 0.3);
            }
        }
        for xx in [-6., 6.] {
            for zz in [-0.9, 0.9] {
                diamond(m, [x + xx, 5.6, z + zz], 0.5);
            }
        }
        for xx in [-1.6, 0., 1.6] {
            diamond(m, [x + xx, 5.55, z + 3.], 0.38);
        }
        diamond(m, [x, 2.5, z + 13.4], 1.1);
        // Wall strips stop short of the rear ramp lane.
        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 13.5, 1.5, z],
                [0.07, 2.5, 15.],
                [0.04, 0.07, 0.11],
                4,
                0.,
            );
        }
    }
    pub(super) fn jewelry_details_15(&self, _m: &mut Mesh) {}
    pub fn jewelry_roof(&self) -> Mesh {
        let mut m = Mesh::default();
        slab(self, &mut m, ROOF);
        for v in &mut m.vertices {
            v.position[1] += self.p.y;
        }
        m
    }
}
pub fn place_0(lib: &mut Library) {
    lib.add(
        "Jewelry",
        CENTER[0],
        CENTER[1],
        [28., 12., 28.],
        Style::Glass,
        true,
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    fn building() -> Building {
        let mut lib = Library::default();
        place_0(&mut lib);
        lib.buildings.remove(0)
    }
    #[test]
    fn ramp_surfaces_reach_each_floor_continuously() {
        let b = building();
        for r in RAMPS {
            let mut previous = r.start[1];
            for i in 0..=200 {
                let t = i as f32 / 200.;
                let p = Vec3::new(
                    b.p.x + r.start[0] + (r.end[0] - r.start[0]) * t,
                    previous + 1.,
                    b.p.z + r.start[2] + (r.end[2] - r.start[2]) * t,
                );
                let h = ramp_height(&b, p).unwrap();
                assert!((h - previous).abs() < 0.05);
                assert_eq!(b.support(p), h);
                previous = h;
            }
            assert!((previous - r.end[1]).abs() < 0.001);
        }
    }
    #[test]
    fn upper_surfaces_never_snap_to_displays_on_lower_floors() {
        let b = building();
        assert_eq!(b.support(Vec3::new(b.p.x + 6., 13., b.p.z)), 12.);
        assert!((b.support(Vec3::new(b.p.x + 6., 9.8, b.p.z + 3.)) - 8.8).abs() < 0.0001);
        assert_eq!(b.support(Vec3::new(b.p.x, 6., b.p.z + 3.)), 5.);
    }
    #[test]
    fn beams_hit_their_visible_positions_and_allow_avoidance() {
        let b = beams(0.)[0];
        assert!(b.touches(Vec3::new(0., 1., -6.), 0));
        assert!(!b.touches(Vec3::new(0., 2., -6.), 0));
        assert!(!b.touches(Vec3::new(11., 1., -6.), 0));
        let high = beams(0.)[6];
        assert!(high.touches(Vec3::new(0., 9., -2.), 0));
        assert!(!high.touches(Vec3::new(0., 9., -2.), 1));
    }
}
