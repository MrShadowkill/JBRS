pub mod airport;
pub mod apartment_rooms;
pub mod apartments;
pub mod bank;
pub mod barracks;
pub mod canteen;
pub mod casino;
pub mod civic_hall;
pub mod criminal_base;
pub mod donut_shop;
pub mod factory;
pub mod garage;
pub mod gas_station;
pub mod grocery;
pub mod gym;
pub mod hangar;
pub mod harbor;
pub mod housing;
pub mod industrial_police;
pub mod infirmary;
pub mod jewelry;
pub mod military;
pub mod museum;
pub mod north_police;
pub mod office;
pub mod police;
pub mod power_plant;
pub mod prison;
pub mod prison_police;
pub mod rail_station;
pub mod showroom;
pub mod skyline;
pub mod substation;
pub mod visitors;
pub mod weapons;
pub mod west_police;
pub mod westhaven;
use engine_core::{Mesh, Vec3};
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Style {
    Glass,
    Brick,
    House,
    Warehouse,
    Civic,
    Neon,
}
#[derive(Clone)]
pub struct Building {
    pub id: usize,
    pub name: &'static str,
    pub p: Vec3,
    pub size: Vec3,
    pub style: Style,
    pub interior: bool,
}
#[derive(Clone)]
pub struct Collider {
    pub p: Vec3,
    pub size: Vec3,
    pub movable: bool,
    pub breakable: bool,
}
impl Collider {
    pub fn contains(&self, p: Vec3, r: f32) -> bool {
        p.y + 1.5 > self.p.y - self.size.y / 2.
            && p.y - 0.8 < self.p.y + self.size.y / 2.
            && p.x > self.p.x - self.size.x / 2. - r
            && p.x < self.p.x + self.size.x / 2. + r
            && p.z > self.p.z - self.size.z / 2. - r
            && p.z < self.p.z + self.size.z / 2. + r
    }
}
impl Building {
    pub fn entrance(&self) -> Vec3 {
        Vec3::new(self.p.x, self.p.y + 1., self.p.z - self.size.z / 2. - 2.)
    }
    pub fn inside(&self, p: Vec3) -> bool {
        (p.x - self.p.x).abs() < self.size.x / 2. && (p.z - self.p.z).abs() < self.size.z / 2.
    }
    pub fn support(&self, p: Vec3) -> f32 {
        if self.name == "Jewelry" {
            if let Some(height) = jewelry::ramp_height(self, p) {
                return height;
            }
            if let Some(height) = jewelry::platform_height(self, p) {
                return height;
            }
        }
        if matches!(self.name, "Jewelry" | "Office" | "Westhaven") {
            return self.p.y
                + ((p.y - self.p.y - 0.6) / 4.)
                    .floor()
                    .max(0.)
                    .min((self.size.y / 4.).floor())
                    * 4.;
        }
        if self.name != "Casino" {
            return self.p.y;
        }
        if p.y > self.p.y + 8.5 {
            return self.p.y + 8.;
        }
        let dx = p.x - self.p.x;
        let dz = p.z - self.p.z;
        let start = -self.size.z * 0.35;
        if (5.5..=8.5).contains(&dx) && (start..=start + 20.).contains(&dz) {
            return (((dz - start) / 1.).floor() + 1.).min(20.) * 0.2;
        }
        if p.y > self.p.y + 4.5 || (dx > 4. && dz > start + 20.) {
            self.p.y + 4.
        } else {
            self.p.y
        }
    }
    pub fn colliders(&self) -> Vec<Collider> {
        let mut out = self.local_colliders();
        for c in &mut out {
            c.p.y += self.p.y;
        }
        out
    }
    fn shell_colliders(&self) -> Vec<Collider> {
        let x = self.p.x;
        let z = self.p.z;
        let w = self.size.x;
        let d = self.size.z;
        let h = self.size.y;
        let c = |p, size| Collider {
            p,
            size,
            movable: false,
            breakable: false,
        };
        if !self.interior {
            return vec![c(Vec3::new(x, h / 2., z), self.size)];
        }
        let mut out = vec![
            c(Vec3::new(x - w / 2., h / 2., z), Vec3::new(0.7, h, d)),
            c(Vec3::new(x + w / 2., h / 2., z), Vec3::new(0.7, h, d)),
            c(Vec3::new(x, h / 2., z + d / 2.), Vec3::new(w, h, 0.7)),
        ];
        for side in [-1., 1.] {
            out.push(c(
                Vec3::new(x + side * (w / 4. + 0.75), h / 2., z - d / 2.),
                Vec3::new(w / 2. - 1.5, h, 0.7),
            ));
        }
        out.push(c(
            Vec3::new(x, (h + 3.) / 2., z - d / 2.),
            Vec3::new(3., h - 3., 0.7),
        ));
        out
    }
    fn local_colliders(&self) -> Vec<Collider> {
        let mut out = self.shell_colliders();
        out.extend(self.crown());
        if self.name == "Jewelry" {
            out.extend(jewelry::colliders(self));
            return out;
        }
        if matches!(self.name, "Apartments" | "Penthouse") {
            out.extend(apartment_rooms::ground_colliders(self));
        }
        if !self.interior {
            return out;
        }
        let x = self.p.x;
        let z = self.p.z;
        let h = self.size.y;
        let c = |p, size| Collider {
            p,
            size,
            movable: false,
            breakable: false,
        };
        if matches!(self.name, "Office" | "Westhaven") {
            for floor in 0..(h / 4.) as usize {
                let y = floor as f32 * 4.;
                for side in [-1., 1.] {
                    for offset in [-1., 1.] {
                        out.push(c(
                            Vec3::new(x + side * 3., y + 1.6, z + offset * 7.5),
                            Vec3::new(0.18, 3.2, 11.),
                        ));
                    }
                }
            }
        }
        if matches!(self.name, "Office" | "Westhaven") {
            out.extend(apartments::furniture_colliders(self));
        }
        if !matches!(self.name, "Office" | "Westhaven") {
            for (p, size, _, _) in self.fittings() {
                out.push(c(p, size));
            }
        }
        out
    }
    pub fn mesh(&self, m: &mut Mesh, lod: u8) {
        let start = m.vertices.len();
        self.local_mesh(m, lod);
        for v in &mut m.vertices[start..] {
            v.position[1] += self.p.y;
        }
    }
    fn local_mesh(&self, m: &mut Mesh, lod: u8) {
        if self.name == "Jewelry" {
            self.jewelry_exterior(m, lod);
            return;
        }
        if self.city_architecture() {
            self.city_mesh(m, lod);
            return;
        }
        let x = self.p.x;
        let z = self.p.z;
        let w = self.size.x;
        let d = self.size.z;
        let h = self.size.y;
        let (mat, c) = match self.style {
            Style::Glass => (3, [0.38, 0.63, 0.73]),
            Style::Brick => (2, [0.74, 0.5, 0.37]),
            Style::House => (12, [0.8, 0.75, 0.58]),
            Style::Warehouse => (4, [0.57, 0.62, 0.6]),
            Style::Civic => (1, [0.81, 0.8, 0.7]),
            Style::Neon => (1, [0.28, 0.3, 0.46]),
        };
        if self.interior {
            for wall in self.shell_colliders() {
                m.cuboid(
                    [wall.p.x, wall.p.y, wall.p.z],
                    [wall.size.x, wall.size.y, wall.size.z],
                    c,
                    mat,
                    0.,
                );
            }
            m.cuboid([x, 0.05, z], [w, 0.1, d], [0.65, 0.65, 0.62], 1, 0.);
        // Roof is a separate interior-occlusion layer, supplied by World when outside.
        } else {
            m.cuboid([x, h / 2., z], [w, h, d], c, mat, 0.);
        }
        m.cuboid(
            [x, if self.interior { -0.2 } else { 0.25 }, z],
            [w + 1., if self.interior { 0.4 } else { 0.5 }, d + 1.],
            [0.5, 0.54, 0.54],
            1,
            0.,
        );
        if self.style == Style::House {
            m.roof([x, h, z], [w + 2., 3., d + 2.], [0.48, 0.29, 0.2]);
            // Keep low-detail homes readable as simple textured boxes: one door
            // and two window panels remain visible without the detailed prop pass.
            m.cuboid(
                [x, 1.15, z - d / 2. - 0.08],
                [1.5, 2.3, 0.12],
                [0.62, 0.38, 0.22],
                2,
                0.,
            );
            for side in [-1., 1.] {
                m.cuboid(
                    [x + side * w * 0.28, 2.65, z - d / 2. - 0.08],
                    [1.9, 1.5, 0.12],
                    [0.55, 0.72, 0.78],
                    3,
                    0.,
                );
            }
            if lod == 0 {
                m.cuboid(
                    [x + w * 0.25, h + 2., z],
                    [1., 4., 1.],
                    [0.6, 0.35, 0.26],
                    2,
                    0.,
                );
            }
        } else {
            for floor in [h, h * 0.45] {
                if self.interior {
                    for side in [-1., 1.] {
                        m.cuboid(
                            [x + side * (w / 2. + 0.25), floor, z],
                            [0.7, 0.5, d + 1.1],
                            [0.65, 0.69, 0.7],
                            4,
                            0.,
                        );
                        m.cuboid(
                            [x, floor, z + side * (d / 2. + 0.25)],
                            [w + 1.1, 0.5, 0.7],
                            [0.65, 0.69, 0.7],
                            4,
                            0.,
                        );
                    }
                } else {
                    m.cuboid(
                        [x, floor, z],
                        [w + 1.1, 0.5, d + 1.1],
                        [0.65, 0.69, 0.7],
                        4,
                        0.,
                    );
                }
            }
            if !self.interior {
                m.cuboid([x, h + 1.5, z], [w * 0.48, 3., d * 0.5], c, mat, 0.);
            }
        }
        if lod > 0 {
            self.sign(m);
            if self.name == "Donut Shop" {
                m.torus([x, h + 3., z], [2.4, 0.65], [0.95, 0.45, 0.65], 8);
            }
            return;
        }
        if self.name == "Donut Shop" {
            m.torus(
                [x, h + 3., z],
                [2.4, 0.65],
                [0.86, 0.42, 0.63],
                if lod == 0 { 24 } else { 8 },
            );
        }
        if matches!(self.name, "Donut Shop" | "Grocery" | "Weapons") {
            let color = match self.name {
                "Donut Shop" => [0.85, 0.38, 0.65],
                "Grocery" => [0.24, 0.65, 0.34],
                _ => [0.22, 0.3, 0.4],
            };
            m.cuboid(
                [x, 4., z - d / 2. - 0.6],
                [w * 0.8, 0.8, 0.4],
                color,
                1,
                0.2,
            );
        }
        self.sign(m);
        if self.name == "Jewelry" {
            for dx in [-w / 2. + 0.7, 0., w / 2. - 0.7] {
                m.cuboid(
                    [x + dx, h / 2., z - d / 2. - 0.04],
                    [0.6, h, 0.08],
                    [0.08, 0.12, 0.2],
                    4,
                    0.,
                );
            }
            for level in 0..3 {
                m.cuboid(
                    [x, 2. + level as f32 * 4., z - d / 2. - 0.025],
                    [w - 2., 3., 0.04],
                    [0.08, 0.5, 0.88],
                    3,
                    0.15,
                );
            }
            for dx in [-w / 2. + 1.5, w / 2. - 1.5] {
                m.cuboid(
                    [x + dx, h / 2., z - d / 2. - 0.09],
                    [0.12, h - 1., 0.04],
                    [0.05, 0.9, 1.],
                    9,
                    0.7,
                );
            }
        }

        if self.name == "Museum" {
            m.roof([x, h, z], [w + 3., 5., d + 3.], [0.72, 0.6, 0.42]);
            for dx in [-15., -5., 5., 15.] {
                m.cylinder(
                    [x + dx, 4., z - d / 2. - 3.],
                    0.8,
                    8.,
                    [0.85, 0.78, 0.62],
                    1,
                    16,
                );
            }
        }
        if self.name == "Power Plant" {
            for side in [-1., 1.] {
                for j in 0..10 {
                    let y = j as f32 * 3.;
                    let rad = 9. - 3. * (j as f32 / 9. * std::f32::consts::PI).sin();
                    m.cylinder(
                        [x + side * 32., y + 1.5, z + 8.],
                        rad,
                        3.,
                        [0.2, 0.25, 0.3],
                        4,
                        if lod == 0 { 20 } else { 8 },
                    );
                }
            }
        }
        if self.name == "Gas Station" {
            for side in [-1., 1.] {
                m.cuboid(
                    [x + side * 7., 1.3, z - d / 2. - 10.],
                    [1.5, 2.6, 1.2],
                    [0.18, 0.7, 0.45],
                    4,
                    0.,
                );
                m.cuboid(
                    [x + side * 12., 2.5, z - d / 2. - 10.],
                    [0.3, 5., 0.3],
                    [0.8, 0.85, 0.85],
                    1,
                    0.,
                );
            }
            m.cuboid(
                [x, 5., z - d / 2. - 10.],
                [27., 0.5, 8.],
                [0.15, 0.8, 0.4],
                4,
                0.,
            );
        }
        if matches!(self.name, "Garage" | "Desert Garage") {
            m.cuboid(
                [x, h + 0.25, z],
                [w + 1., 0.5, d + 1.],
                [0.95, 0.75, 0.06],
                4,
                0.,
            );
        }

        // Repeated windows are compiled into one mesh rather than individual calls.
        let floors = (h / 3.5) as i32;
        let bays = (w / 4.) as i32;
        for floor in 0..floors {
            for bay in 0..bays {
                let wx = x - w / 2. + 2. + bay as f32 * 4.;
                let wy = 2. + floor as f32 * 3.5;
                for side in [-1., 1.] {
                    if self.interior && floor == 0 && (wx - x).abs() < 4. {
                        continue;
                    }
                    let wz = z + side * (d / 2. + 0.02);
                    let lit = (bay + floor + self.id as i32) % 4 == 0;
                    m.cuboid(
                        [wx, wy, wz],
                        [2.4, 1.8, 0.04],
                        if lit {
                            [0.98, 0.84, 0.52]
                        } else {
                            [0.55, 0.75, 0.85]
                        },
                        3,
                        if lit { 0.95 } else { 0. },
                    );
                    if self.style == Style::Brick {
                        m.cuboid(
                            [wx, wy - 1., wz],
                            [2.9, 0.14, 0.35],
                            [0.77, 0.72, 0.61],
                            1,
                            0.,
                        );
                    }
                }
            }
        }
        for side in [-1., 1.] {
            for i in 0..(d / 4.) as i32 {
                m.cuboid(
                    [
                        x + side * (w / 2. + 0.02),
                        2.5,
                        z - d / 2. + 2. + i as f32 * 4.,
                    ],
                    [0.04, 1.8, 2.3],
                    [0.43, 0.62, 0.7],
                    3,
                    0.3,
                );
            }
        }
        m.cuboid(
            [x, 3., z - d / 2. - 1.],
            [4., 0.3, 2.5],
            [0.3, 0.37, 0.45],
            4,
            0.,
        );
        for side in [-1., 1.] {
            m.cuboid(
                [x + side * 1.9, 1.5, z - d / 2. - 1.],
                [0.25, 3., 0.25],
                [0.74, 0.76, 0.72],
                1,
                0.,
            );
        }
        m.cuboid(
            [x, 4., z - d / 2. - 0.6],
            [w * 0.65, 0.65, 0.2],
            if self.style == Style::Neon {
                [0.75, 0.3, 0.9]
            } else {
                [0.18, 0.35, 0.47]
            },
            9,
            0.7,
        );
        if !self.interior && self.style != Style::House {
            m.cuboid(
                [x - w * 0.2, h + 1., z],
                [3., 2., 2.],
                [0.5, 0.52, 0.51],
                4,
                0.,
            );
            m.cylinder([x + w * 0.2, h, z], 0.7, 3., [0.5, 0.54, 0.5], 4, 8);
        }
    }
    // One description drives both visible furniture/walls and collision.
    fn fittings(&self) -> Vec<(Vec3, Vec3, [f32; 3], u8)> {
        let mut out = Vec::new();
        let x = self.p.x;
        let z = self.p.z;
        let w = self.size.x;
        let d = self.size.z;
        let mut add = |dx, y, dz, sx, sy, sz, color, mat| {
            out.push((
                Vec3::new(x + dx, y, z + dz),
                Vec3::new(sx, sy, sz),
                color,
                mat,
            ))
        };
        if self.name == "Weapons" {
            for kind in 0..3 {
                add(
                    -6. + kind as f32 * 5.,
                    0.55,
                    5.,
                    3.,
                    1.1,
                    1.2,
                    [0.2, 0.24, 0.3],
                    4,
                );
            }
        } else if matches!(self.name, "Grocery" | "Gas Station") {
            for side in [-1., 1.] {
                for i in 0..3 {
                    let xx = side * w * 0.32;
                    let zz = -d * 0.28 + i as f32 * d * 0.25;
                    for dx in [-w * 0.09, w * 0.09] {
                        for dz in [-0.92, 0.92] {
                            add(xx + dx, 1.2, zz + dz, 0.08, 2.4, 0.08, [0.36, 0.4, 0.41], 4);
                        }
                    }
                    for row in 0..3 {
                        add(
                            xx,
                            0.8 + row as f32 * 0.6,
                            zz,
                            w * 0.2,
                            0.1,
                            2.,
                            [0.7, 0.72, 0.67],
                            4,
                        );
                    }
                }
            }
        } else if self.name == "Cell Block" {
            for side in [-1., 1.] {
                for i in 0..4 {
                    let zz = -d / 2. + 5. + i as f32 * 10.;
                    add(
                        side * 9.,
                        1.4,
                        zz + 4.5,
                        9.,
                        2.8,
                        0.3,
                        [0.65, 0.67, 0.62],
                        1,
                    );
                    // Split barred cell front around a two metre open doorway.
                    for offset in [-3., 3.] {
                        add(
                            side * 4.5,
                            1.4,
                            zz + offset,
                            0.2,
                            2.8,
                            4.,
                            [0.28, 0.33, 0.36],
                            4,
                        );
                    }
                    add(
                        side * 11.,
                        0.4,
                        zz - 1.,
                        2.,
                        0.8,
                        3.5,
                        [0.28, 0.42, 0.58],
                        10,
                    );
                    add(side * 11., 0.9, zz - 3., 1., 1.8, 0.6, [0.6, 0.62, 0.57], 4);
                }
            }
        } else if matches!(
            self.name,
            "Police"
                | "West Police"
                | "North Police"
                | "Industrial Police"
                | "Prison Police"
                | "Visitors"
                | "Infirmary"
                | "Canteen"
                | "Grocery"
                | "Donut Shop"
        ) {
            for side in [-1., 1.] {
                for i in 0..3 {
                    let zz = -d * 0.28 + i as f32 * d * 0.25;
                    let color = if self.name == "Infirmary" {
                        [0.82, 0.85, 0.84]
                    } else {
                        [0.37, 0.48, 0.58]
                    };
                    add(side * w * 0.32, 0.55, zz, w * 0.2, 1.1, 2., color, 10);
                    if self.name.contains("Police") {
                        add(
                            side * (w / 2. - 1.2),
                            1.25,
                            zz,
                            1.,
                            2.5,
                            1.4,
                            [0.24, 0.31, 0.4],
                            4,
                        );
                    }
                }
            }
        } else {
            for side in [-1., 1.] {
                if self.name == "Casino" && side > 0. {
                    continue;
                }
                add(
                    side * w * 0.32,
                    1.,
                    d * 0.15,
                    w * 0.22,
                    2.,
                    1.5,
                    [0.43, 0.3, 0.22],
                    10,
                );
            }
        }
        out
    }
    pub fn interior_mesh(&self) -> Mesh {
        let mut m = self.local_interior_mesh();
        for v in &mut m.vertices {
            v.position[1] += self.p.y;
        }
        m
    }
    pub fn rendered_floor(&self, p: Vec3) -> usize {
        if matches!(self.name, "Office" | "Westhaven") {
            (((self.support(p) - self.p.y) / 4.).round() as usize)
                .min((self.size.y / 4.) as usize - 1)
        } else {
            0
        }
    }
    pub fn interior_mesh_at(&self, floor: usize) -> Mesh {
        if !matches!(self.name, "Office" | "Westhaven") {
            return self.interior_mesh();
        }
        let mut m = Mesh::default();
        self.expansion_interior(&mut m, Some(floor));
        let floor = floor.min((self.size.y / 4.) as usize - 1);
        m.cuboid(
            [
                self.p.x,
                (floor as f32 * 4. + 4.).min(self.size.y),
                self.p.z,
            ],
            [self.size.x, 0.12, self.size.z],
            [0.72, 0.72, 0.68],
            1,
            0.,
        );
        for v in &mut m.vertices {
            v.position[1] += self.p.y;
        }
        m
    }
    fn local_interior_mesh(&self) -> Mesh {
        let mut m = Mesh::default();
        if self.name == "Jewelry" {
            self.jewelry_interior(&mut m);
            return m;
        }
        let x = self.p.x;
        let z = self.p.z;
        let w = self.size.x;
        let d = self.size.z;
        for (p, size, color, material) in self.fittings() {
            if self.name == "Cell Block" && size.x == 0.2 {
                for i in 0..9 {
                    m.cuboid(
                        [p.x, p.y, p.z - size.z / 2. + i as f32 * 0.5],
                        [0.12, size.y, 0.12],
                        color,
                        material,
                        0.,
                    );
                }
                for y in [0.15, 2.75] {
                    m.cuboid([p.x, y, p.z], [0.18, 0.15, size.z], color, material, 0.);
                }
            } else {
                m.cuboid(
                    [p.x, p.y, p.z],
                    [size.x, size.y, size.z],
                    color,
                    material,
                    0.,
                );
            }
        }
        if self.name == "Cell Block" {
            self.prison_details_0(&mut m);
        }
        for i in 0..3 {
            m.cuboid(
                [x, 4., z - d * 0.3 + i as f32 * d * 0.3],
                [2., 0.08, 0.8],
                [1., 0.9, 0.65],
                9,
                1.,
            );
        }
        // Detailed fittings stay local to this building's elevation.
        m.cuboid(
            [x, 0.025, z],
            [w - 0.8, 0.05, d - 0.8],
            [0.72, 0.71, 0.65],
            3,
            0.,
        );
        if matches!(self.name, "Grocery" | "Donut Shop" | "Weapons" | "Bank") {
            self.grocery_details_1(&mut m);
        }
        if matches!(self.name, "Grocery" | "Gas Station") {
            self.grocery_details_2(&mut m);
        }
        if self.name == "Donut Shop" {
            self.donut_shop_details_3(&mut m);
        }
        if self.name == "Garage" || self.name == "Desert Garage" {
            self.garage_details_4(&mut m);
        }
        if self.name == "Criminal Base" || self.name == "Safehouse" {
            self.criminal_base_details_5(&mut m);
        }
        if self.name == "Gym" {
            self.gym_details_6(&mut m);
        }
        if matches!(self.name, "Apartments" | "Penthouse" | "Safehouse") {
            self.housing_details_7(&mut m);
        }
        if self.name == "Factory" {
            self.factory_details_8(&mut m);
        }
        if self.name == "Weapons" {
            self.weapons_details_9(&mut m);
        }
        if self.name == "Casino" {
            self.casino_details_10(&mut m);
        }
        if matches!(self.name, "Grocery" | "Gas Station") {
            self.grocery_details_11(&mut m);
        }
        if self.name == "Donut Shop" {
            self.donut_shop_details_12(&mut m);
        }
        if self.name == "Bank" {
            self.bank_details_13(&mut m);
        }
        if self.name == "Bank" {
            self.bank_details_14(&mut m);
        }
        if self.name == "Jewelry" {
            self.jewelry_details_15(&mut m);
        }
        self.expansion_interior(&mut m, None);
        m
    }
    pub fn roof_mesh(&self) -> Mesh {
        if self.name == "Jewelry" {
            return self.jewelry_roof();
        }
        let mut m = Mesh::default();
        m.cuboid(
            [self.p.x, self.p.y + self.size.y, self.p.z],
            [self.size.x, 0.4, self.size.z],
            [0.48, 0.51, 0.54],
            4,
            0.,
        );
        m.cuboid(
            [self.p.x, self.p.y + self.size.y - 0.25, self.p.z],
            [self.size.x, 0.1, self.size.z],
            [0.81, 0.82, 0.77],
            1,
            0.2,
        );
        m
    }
}
#[derive(Default)]
pub struct Library {
    pub buildings: Vec<Building>,
}
impl Library {
    pub fn add(
        &mut self,
        name: &'static str,
        x: f32,
        z: f32,
        size: [f32; 3],
        style: Style,
        interior: bool,
    ) {
        let id = self.buildings.len();
        self.buildings.push(Building {
            id,
            name,
            p: Vec3::new(x, 0., z),
            size: Vec3::new(size[0], size[1], size[2]),
            style,
            interior,
        });
    }
}

impl Building {
    fn sign(&self, m: &mut Mesh) {
        if !self.interior {
            return;
        }
        let x = self.p.x;
        let z = self.p.z - self.size.z / 2. - 0.12;
        let y = 5.;
        let c = [0.15, 0.85, 1.];
        match self.name {
            "Jewelry" => {
                m.quad(
                    [
                        [x - 1.5, y, z],
                        [x, y + 2., z],
                        [x + 1.5, y, z],
                        [x, y - 2., z],
                    ],
                    c,
                    9,
                    0.8,
                );
            }
            "Weapons" => {
                m.cuboid([x, y, z], [4., 0.7, 0.08], c, 9, 0.8);
                m.cuboid([x - 1., y - 1., z], [0.7, 1.4, 0.08], c, 9, 0.8);
            }
            "Donut Shop" => {}
            "Bank" | "Casino" => {
                m.price(
                    [x - 1., y - 0.5, z],
                    0.2,
                    if self.name == "Bank" { 100 } else { 777 },
                );
            }
            "Garage" | "Desert Garage" => {
                m.cuboid([x, y, z], [4., 1., 0.08], c, 9, 0.7);
                for dx in [-1.2, 1.2] {
                    m.cuboid(
                        [x + dx, y - 0.7, z],
                        [0.8, 0.8, 0.08],
                        [0.1, 0.15, 0.2],
                        4,
                        0.,
                    );
                }
            }
            "Grocery" => {
                m.cuboid([x, y, z], [2.6, 1.2, 0.06], [0.2, 0.8, 0.3], 9, 0.6);
                for dx in [-0.8, 0., 0.8] {
                    m.cuboid([x + dx, y + 0.8, z], [0.4, 0.8, 0.06], c, 9, 0.6);
                }
            }
            "Gas Station" => {
                m.cuboid([x, y, z], [1.5, 2., 0.06], c, 9, 0.6);
                m.cuboid([x + 1., y, z], [0.2, 1.8, 0.06], c, 9, 0.6);
                m.cuboid(
                    [x, y + 0.3, z - 0.01],
                    [1., 0.6, 0.07],
                    [0.1, 0.2, 0.22],
                    4,
                    0.,
                );
            }
            "Museum" => {
                m.cuboid([x, y, z], [3., 0.4, 0.06], c, 9, 0.6);
                for dx in [-1.5, 1.5] {
                    m.cuboid([x + dx, y, z], [0.7, 1., 0.06], c, 9, 0.6);
                }
            }
            "Power Plant" | "Substation" => {
                for i in 0..3 {
                    let a = i as f32 * std::f32::consts::TAU / 3.;
                    m.triangle(
                        [
                            [x, y, z],
                            [x + a.cos() * 1.8, y + a.sin() * 1.8, z],
                            [x + (a + 0.8).cos() * 1.8, y + (a + 0.8).sin() * 1.8, z],
                        ],
                        [0.95, 0.8, 0.1],
                        9,
                        0.8,
                    );
                }
            }
            "Gym" => {
                m.cuboid([x, y, z], [3., 0.3, 0.06], c, 9, 0.6);
                for dx in [-1.3, 1.3] {
                    m.cuboid([x + dx, y, z], [0.4, 1.5, 0.06], c, 9, 0.6);
                }
            }
            name if name.contains("Police") || name == "Military" => {
                for i in 0..5 {
                    let a = i as f32 * std::f32::consts::TAU / 5.;
                    m.triangle(
                        [
                            [x, y, z],
                            [x + a.cos() * 1.8, y + a.sin() * 1.8, z],
                            [x + (a + 0.6).cos() * 0.8, y + (a + 0.6).sin() * 0.8, z],
                        ],
                        c,
                        9,
                        0.7,
                    );
                }
            }
            _ => {
                m.cuboid([x, y, z], [3., 1.3, 0.06], c, 9, 0.6);
                m.cuboid([x, y + 0.8, z], [2., 0.6, 0.06], c, 9, 0.6);
            }
        }
    }
    fn expansion_interior(&self, m: &mut Mesh, selected: Option<usize>) {
        if matches!(self.name, "Office" | "Westhaven") {
            self.apartments_interior(m, selected);
        }
        if self.name == "Bank" {
            self.bank_interior(m);
        }
        if self.name == "Jewelry" {
            self.jewelry_interior(m);
        }
        if self.name == "Casino" {
            self.casino_interior(m);
        }
        if self.name == "Museum" {
            self.museum_interior(m);
        }
        if self.name == "Power Plant" {
            self.power_plant_interior(m);
        }
        if self.name == "Gas Station" {
            self.gas_station_interior(m);
        }
    }
}

fn museum_sphere(m: &mut Mesh, p: [f32; 3], radius: f32, c: [f32; 3]) {
    for i in 0..12 {
        for j in 0..6 {
            let point = |u: f32, v: f32| {
                let a = u / 12. * std::f32::consts::TAU;
                let b = v / 6. * std::f32::consts::PI;
                [
                    p[0] + radius * a.cos() * b.sin(),
                    p[1] + radius * b.cos(),
                    p[2] + radius * a.sin() * b.sin(),
                ]
            };
            m.quad(
                [
                    point(i as f32, j as f32),
                    point((i + 1) as f32, j as f32),
                    point((i + 1) as f32, (j + 1) as f32),
                    point(i as f32, (j + 1) as f32),
                ],
                c,
                1,
                0.,
            );
        }
    }
}

fn museum_bone(m: &mut Mesh, a: [f32; 3], b: [f32; 3], radius: f32, color: [f32; 3]) {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(0.001);
    let d = d.map(|x| x / len);
    let raw = if d[0].abs() + d[1].abs() > 0.01 {
        [d[1], -d[0], 0.]
    } else {
        [1., 0., 0.]
    };
    let len = (raw[0] * raw[0] + raw[1] * raw[1] + raw[2] * raw[2]).sqrt();
    let n = raw.map(|x| x / len);
    let t = [
        d[1] * n[2] - d[2] * n[1],
        d[2] * n[0] - d[0] * n[2],
        d[0] * n[1] - d[1] * n[0],
    ];
    for i in 0..8 {
        let point = |p: [f32; 3], j: usize| {
            let f = j as f32 / 8. * std::f32::consts::TAU;
            [
                p[0] + radius * (n[0] * f.cos() + t[0] * f.sin()),
                p[1] + radius * (n[1] * f.cos() + t[1] * f.sin()),
                p[2] + radius * (n[2] * f.cos() + t[2] * f.sin()),
            ]
        };
        m.quad(
            [point(a, i), point(b, i), point(b, i + 1), point(a, i + 1)],
            color,
            1,
            0.,
        );
    }
}
fn museum_gem(m: &mut Mesh, p: [f32; 3], r: f32) {
    for i in 0..4 {
        let a = i as f32 * std::f32::consts::FRAC_PI_2;
        let b = (i + 1) as f32 * std::f32::consts::FRAC_PI_2;
        let q = [p[0] + r * a.cos(), p[1], p[2] + r * a.sin()];
        let w = [p[0] + r * b.cos(), p[1], p[2] + r * b.sin()];
        m.triangle([[p[0], p[1] + r, p[2]], q, w], [0.05, 0.85, 1.], 3, 0.4);
        m.triangle([[p[0], p[1] - r, p[2]], w, q], [0.1, 0.6, 0.95], 3, 0.2);
    }
}
