use super::*;
use buildings::Style;
pub fn build() -> World {
    let mut roadnet = roads::network();
    let mut lib = Library::default();
    let mut colliders = Vec::new();
    let mut props = Vec::new();
    let mut pois = Vec::new();
    let mut add_prop = |p: [f32; 3], size: [f32; 3], color: [f32; 3], mat: u8, solid: bool| {
        let p = Vec3::new(p[0], p[1], p[2]);
        let size = Vec3::new(size[0], size[1], size[2]);
        props.push(Prop {
            p,
            size,
            color,
            material: mat,
            emission: if mat == 9 { 1. } else { 0. },
            roof: false,
        });
        if solid {
            colliders.push(Collider {
                p,
                size,
                movable: mat == 4 && size.x <= 12. && size.z <= 12. && size.y <= 4.,
                breakable: mat == 3,
            });
        }
    };
    // Preserve established game coordinates; the yard is now separate from public roads.
    for (x, z, sx, sz) in [
        (-100., -220., 140., 1.),
        (-170., -169., 1., 102.),
        (-170., -101., 1., 2.),
        (-30., -160., 1., 120.),
        (-140., -100., 60., 1.),
        (-50., -100., 40., 1.),
    ] {
        add_prop([x, 3., z], [sx, 6., sz], [0.45, 0.5, 0.52], 4, true);
    }
    add_prop([-90., 3., -100.], [40., 6., 1.], [0.22, 0.28, 0.3], 4, true);
    // Western weak panel is handled by the dynamic breach state.
    buildings::prison::place_0(&mut lib);
    buildings::canteen::place_0(&mut lib);
    buildings::prison_police::place_0(&mut lib);
    buildings::visitors::place_0(&mut lib);
    buildings::infirmary::place_0(&mut lib);
    buildings::police::place_0(&mut lib);
    for (x, z) in [(-169., -218.), (-31., -218.)] {
        add_prop([x, 5., z], [2., 10., 2.], [0.45, 0.5, 0.5], 1, true);
        add_prop([x, 10.5, z], [7., 3., 7.], [0.6, 0.63, 0.61], 3, false);
        add_prop([x, 12.2, z], [9., 0.4, 9.], [0.3, 0.35, 0.37], 4, false);
    }
    add_prop(
        [-95., 0.02, -183.],
        [26., 0.04, 25.],
        [0.34, 0.42, 0.39],
        1,
        false,
    );
    for x in [-108., -82.] {
        add_prop(
            [x, 2.7, -183.],
            [0.16, 5.4, 0.16],
            [0.55, 0.58, 0.59],
            4,
            false,
        );
        add_prop(
            [x, 5., -183.],
            [1.8, 1.3, 0.12],
            [0.85, 0.88, 0.86],
            1,
            false,
        );
    }
    add_prop(
        [193., 0.006, -80.],
        [40., 0.015, 40.],
        [0.48, 0.46, 0.43],
        1,
        false,
    );
    add_prop(
        [214., 0.015, -80.],
        [18., 0.025, 8.],
        [0.67, 0.64, 0.59],
        1,
        false,
    );
    add_prop(
        [151., 0.006, -80.],
        [66., 0.015, 40.],
        [0.43, 0.43, 0.43],
        0,
        false,
    );
    // Two visible escape routes: damaged south gate and western service corridor.
    add_prop(
        [-95., 3.5, -100.],
        [32., 0.4, 1.],
        [0.57, 0.58, 0.55],
        4,
        false,
    );
    // Street blocks respect 60m pitch and keep bank access clear.
    for (ix, x) in [60., 110., 210., 260., 370., 420.].into_iter().enumerate() {
        for (iz, z) in [50., 100., 190., 240.].into_iter().enumerate() {
            if (x == 260. && z == 190.)
                || (x == 370. && z == 240.)
                || (x == 420. && z == 100.)
                || (x == 60. && z == 240.)
                || (x == 110. && z == 240.)
                || (x == 210. && z == 240.)
            {
                continue;
            }
            let height = if x == 370. && z == 100. {
                104.
            } else if x == 210. && z == 50. {
                88.
            } else {
                24. + ((ix * 13 + iz * 19) % 67) as f32
            };
            buildings::office::place_0(&mut lib, x, z, height, ix);
        }
    }
    buildings::bank::place_0(&mut lib);
    buildings::jewelry::place_0(&mut lib);
    buildings::casino::place_0(&mut lib);
    buildings::showroom::place_0(&mut lib);
    buildings::donut_shop::place_0(&mut lib);
    buildings::grocery::place_0(&mut lib);
    buildings::weapons::place_0(&mut lib);
    buildings::garage::place_1(&mut lib);
    buildings::criminal_base::place_1(&mut lib);
    buildings::criminal_base::place_0(&mut lib);
    buildings::gym::place_0(&mut lib);
    // Fictional fortified detention camp: layered fence, wire and floodlights.
    for x in (-165..-30).step_by(6) {
        for z in [-220., -100.] {
            add_prop(
                [x as f32, 6.35, z],
                [0.1, 0.7, 0.1],
                [0.22, 0.25, 0.24],
                4,
                false,
            );
            add_prop(
                [x as f32, 6.4, z],
                [5.8, 0.08, 0.08],
                [0.28, 0.29, 0.26],
                4,
                false,
            );
        }
    }
    for z in (-215..-100).step_by(6) {
        for x in [-170., -30.] {
            add_prop(
                [x, 6.4, z as f32],
                [0.08, 0.08, 5.8],
                [0.28, 0.29, 0.26],
                4,
                false,
            );
        }
    }
    for (x, z) in [(-120., -215.), (-80., -105.), (-160., -150.), (-35., -175.)] {
        add_prop([x, 4., z], [0.18, 8., 0.18], [0.26, 0.3, 0.28], 4, true);
        add_prop([x, 7.8, z], [2.4, 0.3, 0.6], [0.95, 0.87, 0.62], 9, false);
    }
    for x in [-111., -85.] {
        for z in [-172., -158.] {
            add_prop([x, 0.25, z], [5., 0.5, 1.4], [0.39, 0.35, 0.27], 10, true);
        }
    }
    buildings::north_police::place_0(&mut lib);
    buildings::industrial_police::place_0(&mut lib);
    buildings::housing::place_0(&mut lib);
    add_prop(
        [350., 9., -50.],
        [28., 18., 24.],
        [0.38, 0.52, 0.62],
        3,
        true,
    );
    // Pedestrian infrastructure: crossing stripes, signals, benches, bus stops.
    for x in [0., 160., 480.] {
        for z in [0., 140., 300.] {
            for side in [-1., 1.] {
                for k in 0..6 {
                    add_prop(
                        [x - 3.5 + k as f32 * 1.4, 0.09, z + side * 10.],
                        [0.65, 0.04, 3.],
                        [0.86, 0.89, 0.86],
                        1,
                        false,
                    );
                }
                let xx = x + side * 10.;
                let zz = z + 10.;
                add_prop([xx, 2., zz], [0.15, 4., 0.15], [0.32, 0.35, 0.35], 4, false);
                add_prop(
                    [xx, 4., zz],
                    [0.45, 1.1, 0.35],
                    [0.12, 0.15, 0.15],
                    4,
                    false,
                );
                add_prop(
                    [xx, 4.2, zz - 0.2],
                    [0.2, 0.2, 0.1],
                    [1., 0.2, 0.12],
                    9,
                    false,
                );
            }
        }
    }
    for x in [38., 190., 350., 450.] {
        add_prop(
            [x, 0.6, -13.],
            [3., 0.35, 0.9],
            [0.55, 0.36, 0.22],
            10,
            false,
        );
        add_prop(
            [x + 4., 0.5, -13.],
            [0.8, 1., 0.8],
            [0.26, 0.36, 0.37],
            4,
            false,
        );
        add_prop([x, 2., -17.], [5., 0.25, 2.], [0.4, 0.47, 0.48], 4, false);
        for dx in [-2., 2.] {
            add_prop(
                [x + dx, 1., -17.],
                [0.12, 2., 0.12],
                [0.35, 0.4, 0.42],
                4,
                false,
            );
        }
    }
    // Residential district west of town, individually coloured pitched-roof homes.
    for (i, x) in [-350., -300., -220., -170., -90.].into_iter().enumerate() {
        for z in [210., 260., 410.] {
            let apartment = (x == -220. && z == 210.) || (x == -300. && z == 260.);
            lib.add(
                if apartment { "Apartments" } else { "Residence" },
                x,
                z,
                [20., 5., 22.],
                Style::House,
                apartment,
            );
            add_prop(
                [x, 0.03, z - 20.],
                [5., 0.06, 18.],
                [0.62, 0.62, 0.58],
                1,
                false,
            );
            for side in [-1., 1.] {
                add_prop(
                    [x + side * 15., 0.65, z],
                    [0.15, 1.3, 30.],
                    [0.7, 0.59, 0.4],
                    10,
                    true,
                );
            }
            add_prop(
                [x + 7., 5.5, z + 3.],
                [1., 1., 1.],
                [0.37 + i as f32 * 0.03, 0.35, 0.34],
                4,
                false,
            );
        }
    }
    // Second town: civic waterfront square and brick houses along west-shore road.
    for x in [-760., -620.] {
        for z in [-160., -60., 60., 210.] {
            buildings::westhaven::place_0(&mut lib, x, z);
        }
    }
    buildings::west_police::place_0(&mut lib);
    // Industrial yards southeast; factory, warehouse, power station and pipe racks.
    for x in [560., 640., 720.] {
        buildings::factory::place_0(&mut lib, x);
        for z in [188., 252.] {
            add_prop([x, 8., z], [1., 16., 1.], [0.6, 0.61, 0.58], 4, false);
        }
        for dx in [-12., 12.] {
            add_prop(
                [x + dx, 1.5, 270.],
                [8., 3., 5.],
                [0.32, 0.45, 0.57],
                4,
                true,
            );
        }
    }
    buildings::substation::place_0(&mut lib);
    for x in [545., 570., 595., 620.] {
        add_prop([x, 15., -115.], [3., 30., 3.], [0.5, 0.52, 0.51], 2, true);
        add_prop([x, 2., -55.], [12., 4., 10.], [0.64, 0.66, 0.62], 4, true);
    }
    // Airport southern lowland; open terminal hall, taxiways and runway.
    buildings::airport::place_0(&mut lib);
    add_prop(
        [795., 0.05, 650.],
        [34., 0.1, 420.],
        [0.34, 0.36, 0.38],
        0,
        false,
    );
    for z in (460..850).step_by(22) {
        add_prop(
            [795., 0.12, z as f32],
            [1., 0.05, 12.],
            [0.94, 0.94, 0.89],
            1,
            false,
        );
    }
    buildings::hangar::place_0(&mut lib);
    add_prop(
        [685., 22., 765.],
        [5., 44., 5.],
        [0.52, 0.59, 0.59],
        1,
        true,
    );
    add_prop(
        [685., 43., 765.],
        [12., 5., 12.],
        [0.42, 0.63, 0.7],
        3,
        false,
    );
    // Harbor and original container cranes.
    buildings::harbor::place_0(&mut lib);
    add_prop(
        [895., 0.02, 120.],
        [90., 0.04, 160.],
        [0.5, 0.52, 0.49],
        1,
        false,
    );
    for x in [850., 880., 910.] {
        for z in [70., 100., 145.] {
            add_prop(
                [x, 1.5, z],
                [12., 3., 5.],
                [0.68, 0.31 + ((x as i32 % 30) as f32) / 100., 0.22],
                4,
                true,
            );
        }
    }
    for z in [40., 170.] {
        add_prop([900., 14., z], [2., 28., 2.], [0.93, 0.68, 0.16], 4, true);
        add_prop([900., 28., z], [50., 2., 2.], [0.93, 0.68, 0.16], 4, false);
        add_prop([918., 21., z], [0.1, 14., 0.1], [0.4, 0.42, 0.43], 4, false);
    }
    // Desert settlement and military outpost north-east, connected to desert route.
    buildings::garage::place_0(&mut lib);
    buildings::military::place_0(&mut lib);
    for x in [540., 580., 620.] {
        buildings::barracks::place_0(&mut lib, x);
    }
    // Park fountain, museum landmark and railway station.
    buildings::civic_hall::place_0(&mut lib);
    buildings::rail_station::place_0(&mut lib);
    add_prop(
        [320., 0.05, 225.],
        [30., 0.1, 30.],
        [0.3, 0.48, 0.3],
        6,
        false,
    );
    add_prop(
        [320., 0.7, 225.],
        [9., 1.4, 9.],
        [0.64, 0.66, 0.61],
        1,
        false,
    );
    add_prop(
        [320., 1.45, 225.],
        [7., 0.1, 7.],
        [0.3, 0.61, 0.72],
        8,
        false,
    );
    for (name, x, z, color) in [
        ("Prison", -100., -160., [0.95, 0.5, 0.2]),
        ("Bank", 320., 35., [1., 0.8, 0.2]),
        ("Police", 0., -190., [0.3, 0.5, 1.]),
        ("Criminal Base", -290., 100., [1., 0.25, 0.2]),
        ("Downtown", 250., 100., [0.65, 0.8, 0.9]),
        ("Westhaven", -680., 60., [0.75, 0.7, 0.55]),
        ("Airport", 700., 720., [0.7, 0.8, 0.9]),
        ("Harbor", 865., 200., [0.7, 0.8, 0.9]),
        ("Mountains", 180., -650., [0.7, 0.7, 0.65]),
        ("Desert", 700., -450., [0.9, 0.7, 0.4]),
        ("Industrial", 640., 220., [0.6, 0.65, 0.7]),
        ("Jewelry", 260., 190., [0.4, 0.9, 0.9]),
        ("Casino", 370., 240., [0.85, 0.35, 0.95]),
        ("Garage", 430., -45., [0.5, 0.8, 0.6]),
        ("Military", 585., -500., [0.6, 0.6, 0.3]),
        ("Weapons", 60., 240., [0.8, 0.6, 0.5]),
        (
            "Donut Shop",
            buildings::donut_shop::CENTER.x,
            buildings::donut_shop::CENTER.z,
            [0.9, 0.45, 0.7],
        ),
        ("Grocery", 210., 240., [0.3, 0.85, 0.4]),
        ("Safehouse", -335., 320., [0.9, 0.2, 0.2]),
        ("Gym", -260., 340., [0.7, 0.65, 0.3]),
        ("North Police", 450., 410., [0.2, 0.5, 1.]),
        ("Industrial Police", 700., 90., [0.2, 0.5, 1.]),
        ("Apartments", -220., 210., [0.8, 0.65, 0.3]),
        ("Penthouse", 350., -50., [0.8, 0.65, 0.3]),
    ] {
        pois.push(Poi {
            name,
            p: Vec3::new(x, 0., z),
            color,
        });
    }
    let mut trees = Vec::new();
    let mut seed = 0x79ab_23d1_67f0_951bu64;
    let mut random = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 32) as u32 as f32 / u32::MAX as f32
    };
    for _ in 0..3200 {
        if trees.len() >= 900 {
            break;
        }
        let p = Vec3::new(-940. + random() * 1880., 0., -940. + random() * 1880.);
        if terrain::height(p.x, p.z) < -0.2
            || roadnet.iter().any(|r| r.distance(p).0 < r.width / 2. + 9.)
            || lib.buildings.iter().any(|b| {
                (p.x - b.p.x).abs() < b.size.x / 2. + 12.
                    && (p.z - b.p.z).abs() < b.size.z / 2. + 12.
            })
            || trees
                .iter()
                .any(|t: &Vec3| (p.x - t.x).hypot(p.z - t.z) < 14.)
        {
            continue;
        }
        trees.push(p);
    }
    for p in [
        (-12., 45.),
        (-12., 95.),
        (145., 190.),
        (145., 240.),
        (465., 45.),
        (465., 95.),
        (190., -12.),
        (240., -12.),
        (290., -12.),
        (-200., 185.),
        (-320., 185.),
    ] {
        trees.push(Vec3::new(
            p.0 + (random() - 0.5) * 9.,
            0.,
            p.1 + (random() - 0.5) * 9.,
        ));
    }
    for b in &mut lib.buildings {
        b.p.y =
            terrain::height(b.p.x, b.p.z).max(0.) + if b.name == "Penthouse" { 18. } else { 0. };
    }
    for p in &mut props {
        p.p.y += terrain::height(p.p.x, p.p.z).max(0.);
    }
    for c in &mut colliders {
        c.p.y += terrain::height(c.p.x, c.p.z).max(0.);
    }
    buildings::museum::place_0(&mut lib);
    buildings::power_plant::place_0(&mut lib);
    buildings::gas_station::place_0(&mut lib);
    for b in &mut lib.buildings {
        if matches!(b.name, "Museum" | "Power Plant" | "Gas Station") {
            b.p.y = terrain::height(b.p.x, b.p.z).max(0.);
        }
    }
    for name in ["Museum", "Power Plant", "Gas Station"] {
        let b = lib.buildings.iter().find(|b| b.name == name).unwrap();
        pois.push(Poi {
            name,
            p: b.p,
            color: [0.2, 0.8, 0.8],
        });
    }
    let mut points = vec![Vec3::new(160., 0., -300.)];
    for i in 1..=42 {
        let t = i as f32 / 42.;
        points.push(Vec3::new(160. - 60. * t, 0., -300. - 330. * t));
    }
    for i in 1..=11 {
        points.push(Vec3::new(100. - 82. * i as f32 / 11., 0., -630.));
    }
    for i in 0..=240 {
        let t = i as f32 / 240.;
        let angle = t * std::f32::consts::TAU * 2.;
        let radius = 180. * (1. - t) + 38.;
        let x = -200. + angle.cos() * radius;
        let z = -630. + angle.sin() * radius;
        points.push(Vec3::new(x, terrain::height(x, z).max(0.), z));
    }
    let museum = lib.buildings.iter().find(|b| b.name == "Museum").unwrap();
    let mut entry = museum.entrance();
    entry.z -= 6.;
    entry.y = museum.p.y;
    points.push(Vec3::new(-162., museum.p.y, entry.z));
    points.push(entry);
    // A continuous gentle ascent, including the final museum approach.
    let distances: Vec<f32> = std::iter::once(0.)
        .chain(points.windows(2).scan(0., |sum, p| {
            *sum += p[0].distance(p[1]);
            Some(*sum)
        }))
        .collect();
    let total = distances.last().copied().unwrap_or(1.);
    for (p, d) in points.iter_mut().zip(distances) {
        p.y = museum.p.y * d / total;
        // Flat landings keep the entry/exit ribbons from overlapping at different heights.
        let start = ((d - 45.) / 45.).clamp(0., 1.);
        p.y *= start;
        let to_end = total - d;
        if to_end < 240. {
            p.y = p.y.max(museum.p.y - (to_end - 200.).max(0.) * 0.08);
        }
    }
    roadnet.push(Road {
        name: "Museum spiral",
        points,
        width: 8.,
        highway: false,
    });
    city_blocks::infill(&roadnet, &mut lib, &props);
    trees.retain(|t| {
        if (118. ..214.).contains(&t.x) && (-102. ..-58.).contains(&t.z) {
            return false;
        }
        if terrain::railway::nearest(*t).0 < 9. {
            return false;
        }
        roadnet.iter().all(|r| r.distance(*t).0 > r.width / 2. + 8.)
            && lib.buildings.iter().all(|b| {
                (t.x - b.p.x).abs() > b.size.x / 2. + 12.
                    || (t.z - b.p.z).abs() > b.size.z / 2. + 12.
            })
    });
    // Low polygon street lamps on ordinary city roads. Their bulb tiles glow
    // at night in both renderers; poles stay outside the carriageway.
    let mut street_lamps = 0usize;
    'roads: for road in roadnet
        .iter()
        .filter(|road| !road.highway && road.width <= 24.)
    {
        let mut next_lamp = 34.0f32;
        let mut walked = 0.0f32;
        for segment in road.points.windows(2) {
            let a = segment[0];
            let b = segment[1];
            let dx = b.x - a.x;
            let dz = b.z - a.z;
            let length = dx.hypot(dz);
            if length < 0.1 {
                continue;
            }
            while next_lamp <= walked + length {
                let t = (next_lamp - walked) / length;
                let side = if street_lamps.is_multiple_of(2) {
                    1.0
                } else {
                    -1.0
                };
                let nx = -dz / length * side;
                let nz = dx / length * side;
                let offset = road.width * 0.5 + 4.5;
                let x = a.x + dx * t + nx * offset;
                let z = a.z + dz * t + nz * offset;
                let near_building = lib.buildings.iter().any(|building| {
                    (x - building.p.x).abs() < building.size.x * 0.5 + 2.0
                        && (z - building.p.z).abs() < building.size.z * 0.5 + 2.0
                });
                let in_road = roadnet
                    .iter()
                    .any(|other| other.distance(Vec3::new(x, 0., z)).0 < other.width * 0.5 + 3.);
                if !near_building
                    && !in_road
                    && terrain::railway::nearest(Vec3::new(x, 0., z)).0 > 7.0
                    && x.abs() < 930.0
                    && z.abs() < 930.0
                {
                    let ground = terrain::height(x, z).max(0.0);
                    let pole = Vec3::new(x, ground + 3.8, z);
                    props.push(Prop {
                        p: pole,
                        size: Vec3::new(0.16, 7.6, 0.16),
                        material: 4,
                        color: [0.28, 0.3, 0.31],
                        emission: 0.,
                        roof: false,
                    });
                    colliders.push(Collider {
                        p: pole,
                        size: Vec3::new(0.16, 7.6, 0.16),
                        movable: false,
                        breakable: false,
                    });
                    props.push(Prop {
                        p: Vec3::new(x, ground + 7.55, z),
                        size: Vec3::new(1.15, 0.2, 0.42),
                        material: 9,
                        color: [1.0, 0.78, 0.46],
                        emission: 0.95,
                        roof: false,
                    });
                    street_lamps += 1;
                    if street_lamps >= 180 {
                        break 'roads;
                    }
                }
                next_lamp += 72.0;
            }
            walked += length;
        }
    }
    World::assemble(roadnet, lib, colliders, props, pois, trees)
}
