use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn distance(self, b: Self) -> f32 {
        ((self.x - b.x).powi(2) + (self.z - b.z).powi(2)).sqrt()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Team {
    Prisoner,
    Criminal,
    Police,
}
pub const STEP: f32 = 1.0 / 60.0;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}
#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
}
impl Mesh {
    /// Price labels built into the static shop mesh, readable in the world.
    pub fn price(&mut self, p: [f32; 3], scale: f32, value: u32) {
        let digits = [
            [14, 17, 19, 21, 25, 17, 14],
            [4, 12, 4, 4, 4, 4, 14],
            [14, 17, 1, 2, 4, 8, 31],
            [30, 1, 1, 14, 1, 1, 30],
            [2, 6, 10, 18, 31, 2, 2],
            [31, 16, 16, 30, 1, 1, 30],
            [14, 16, 16, 30, 17, 17, 14],
            [31, 1, 2, 4, 8, 8, 8],
            [14, 17, 17, 14, 17, 17, 14],
            [14, 17, 17, 15, 1, 1, 14],
        ];
        for (i, ch) in format!("${value}").chars().enumerate() {
            let rows = if ch == '$' {
                [4, 15, 20, 14, 5, 30, 4]
            } else {
                digits[ch.to_digit(10).unwrap() as usize]
            };
            for (y, row) in rows.iter().enumerate() {
                for x in 0..5 {
                    if row & (1 << (4 - x)) != 0 {
                        self.cuboid(
                            [
                                p[0] + format!("${value}").len() as f32 * 6. * scale
                                    - (i * 6 + x) as f32 * scale,
                                p[1] - y as f32 * scale,
                                p[2],
                            ],
                            [scale, scale, 0.006],
                            [0.1, 0.12, 0.1],
                            9,
                            0.25,
                        );
                    }
                }
            }
        }
    }
    pub fn triangle(&mut self, p: [[f32; 3]; 3], c: [f32; 3], material: u8, emission: f32) {
        let tx = (material % 4) as f32 / 4.;
        let ty = (material / 4) as f32 / 4.;
        for (i, position) in p.into_iter().enumerate() {
            self.vertices.push(Vertex {
                position,
                color: [c[0], c[1], c[2], emission],
                uv: [
                    tx + if i == 1 { 0.245 } else { 0.005 },
                    ty + if i == 2 { 0.245 } else { 0.005 },
                ],
            });
        }
    }
    pub fn quad(&mut self, p: [[f32; 3]; 4], c: [f32; 3], mat: u8, e: f32) {
        let tx = (mat % 4) as f32 / 4.;
        let ty = (mat / 4) as f32 / 4.;
        for i in [0, 1, 2, 0, 2, 3] {
            let uv = match i {
                0 => [0.005, 0.005],
                1 => [0.245, 0.005],
                2 => [0.245, 0.245],
                _ => [0.005, 0.245],
            };
            self.vertices.push(Vertex {
                position: p[i],
                color: [c[0], c[1], c[2], e],
                uv: [tx + uv[0], ty + uv[1]],
            });
        }
    }
    pub fn cuboid(&mut self, p: [f32; 3], s: [f32; 3], c: [f32; 3], mat: u8, e: f32) {
        let [x, y, z] = p;
        let [a, b, d] = s.map(|v| v / 2.);
        let v = [
            [x - a, y - b, z - d],
            [x + a, y - b, z - d],
            [x + a, y + b, z - d],
            [x - a, y + b, z - d],
            [x - a, y - b, z + d],
            [x + a, y - b, z + d],
            [x + a, y + b, z + d],
            [x - a, y + b, z + d],
        ];
        for (f, k) in [
            ([0, 1, 2, 3], 0.72),
            ([5, 4, 7, 6], 0.84),
            ([4, 0, 3, 7], 0.76),
            ([1, 5, 6, 2], 0.9),
            ([3, 2, 6, 7], 1.),
            ([4, 5, 1, 0], 0.6),
        ] {
            self.quad(f.map(|i| v[i]), c.map(|v| v * k), mat, e);
        }
    }
    pub fn torus(&mut self, p: [f32; 3], radii: [f32; 2], color: [f32; 3], n: usize) {
        let n = n.clamp(8, 24);
        for i in 0..n {
            for j in 0..6 {
                let v = |ii: usize, jj: usize| {
                    let a = std::f32::consts::TAU * ii as f32 / n as f32;
                    let b = std::f32::consts::TAU * jj as f32 / 6.;
                    let radius = radii[0] + radii[1] * b.cos();
                    [
                        p[0] + radius * a.cos(),
                        p[1] + radius * a.sin(),
                        p[2] + radii[1] * b.sin(),
                    ]
                };
                self.quad(
                    [v(i, j), v(i + 1, j), v(i + 1, j + 1), v(i, j + 1)],
                    color,
                    1,
                    0.,
                );
            }
        }
    }
    pub fn cylinder(&mut self, p: [f32; 3], radius: f32, h: f32, c: [f32; 3], mat: u8, n: usize) {
        for i in 0..n {
            let a = i as f32 * std::f32::consts::TAU / n as f32;
            let b = (i + 1) as f32 * std::f32::consts::TAU / n as f32;
            let pa = [p[0] + a.cos() * radius, p[1], p[2] + a.sin() * radius];
            let pb = [p[0] + b.cos() * radius, p[1], p[2] + b.sin() * radius];
            self.quad(
                [pa, pb, [pb[0], p[1] + h, pb[2]], [pa[0], p[1] + h, pa[2]]],
                c.map(|v| v * (0.8 + 0.15 * a.cos())),
                mat,
                0.,
            );
            self.triangle(
                [
                    [p[0], p[1] + h, p[2]],
                    [pa[0], p[1] + h, pa[2]],
                    [pb[0], p[1] + h, pb[2]],
                ],
                c,
                mat,
                0.,
            );
        }
    }
    pub fn roof(&mut self, position: [f32; 3], size: [f32; 3], c: [f32; 3]) {
        let [x, y, z] = position;
        let [w, rise, d] = size;
        self.quad(
            [
                [x - w / 2., y, z - d / 2.],
                [x + w / 2., y, z - d / 2.],
                [x + w / 2., y + rise, z],
                [x - w / 2., y + rise, z],
            ],
            c,
            11,
            0.,
        );
        self.quad(
            [
                [x - w / 2., y + rise, z],
                [x + w / 2., y + rise, z],
                [x + w / 2., y, z + d / 2.],
                [x - w / 2., y, z + d / 2.],
            ],
            c.map(|v| v * 0.8),
            11,
            0.,
        );
        for side in [-1., 1.] {
            self.triangle(
                [
                    [x + side * w / 2., y, z - d / 2.],
                    [x + side * w / 2., y, z + d / 2.],
                    [x + side * w / 2., y + rise, z],
                ],
                c,
                11,
                0.,
            );
        }
    }
}
