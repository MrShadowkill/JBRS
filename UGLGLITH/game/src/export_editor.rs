//! Export actual world geometry at LOD0, without renderer distance/frustum culling.
use engine_core::Mesh;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};
fn write_mesh(path: &Path, mesh: Mesh) -> io::Result<usize> {
    let mut file = io::BufWriter::new(fs::File::create(path)?);
    for vertex in &mesh.vertices {
        for value in vertex.position.into_iter().chain(vertex.color) {
            file.write_all(&value.to_le_bytes())?;
        }
    }
    file.flush()?;
    Ok(mesh.vertices.len() / 3)
}
pub(super) fn export(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    let mut game = super::Game::from_save(Default::default())?;
    game.heists = crate::heists::Heists::new(0);
    let world = &game.world;
    world.audit()?;
    let mut manifest = io::BufWriter::new(fs::File::create(dir.join("meshes.tsv"))?);
    for x in -8..8 {
        for z in -8..8 {
            let name = format!("chunk_{x}_{z}");
            let count = write_mesh(&dir.join(format!("{name}.bin")), world.chunk((x, z), 0))?;
            writeln!(manifest, "{name}\tWorld/{x},{z}\t{count}")?;
        }
    }
    for b in world.buildings.iter().filter(|b| b.interior) {
        let name = format!("interior_{}", b.id);
        let count = write_mesh(&dir.join(format!("{name}.bin")), b.interior_mesh())?;
        writeln!(manifest, "{name}\tInteriors/{} {}\t{count}", b.name, b.id)?;
        let name = format!("roof_{}", b.id);
        let count = write_mesh(&dir.join(format!("{name}.bin")), b.roof_mesh())?;
        writeln!(manifest, "{name}\tRoofs/{} {}\t{count}", b.name, b.id)?;
    }
    let count = write_mesh(&dir.join("sewer.bin"), world::prison::sewer_mesh())?;
    writeln!(manifest, "sewer\tPrison/Sewer\t{count}")?;
    let mut groups = io::BufWriter::new(fs::File::create(dir.join("buildings.tsv"))?);
    for b in &world.buildings {
        writeln!(
            groups,
            "{}\t{}\t{}\t{}",
            b.id,
            b.name,
            (b.p.x / 128.).floor() as i32,
            (b.p.z / 128.).floor() as i32
        )?;
        let mut mesh = Mesh::default();
        b.mesh(&mut mesh, 0);
        write_mesh(&dir.join(format!("building_{}.bin", b.id)), mesh)?;
    }
    for (id, car) in game.cars.iter().enumerate() {
        for (part, source, tint) in [
            ("Paint", &game.car_models[car.kind][1].paint, car.color),
            ("Trim", &game.car_models[car.kind][1].trim, [1.; 3]),
        ] {
            let mesh = transformed(source, car.pos, car.angle, tint);
            let name = format!("car_{id}_{part}");
            let count = write_mesh(&dir.join(format!("{name}.bin")), mesh)?;
            writeln!(
                manifest,
                "{name}\tCars/{} {}\t{count}",
                vehicles::CATALOG[car.kind].name,
                id
            )?;
        }
    }
    let (p, a) = terrain::railway::locomotive(game.train.distance);
    let count = write_mesh(
        &dir.join("locomotive.bin"),
        transformed(&vehicles::rail::locomotive(true), p, a, [1.; 3]),
    )?;
    writeln!(manifest, "locomotive\tTrain/Ironvale D90\t{count}")?;
    for wagon in 0..25 {
        let (p, a) = terrain::railway::carriage(game.train.distance, wagon);
        let mut mesh = vehicles::rail::running_gear();
        wagon_chassis(&mut mesh);
        mesh.vertices
            .extend(vehicles::rail::wagon_equipment().vertices);
        mesh.cuboid(
            [0., 1.8, 0.],
            [3.2, 2.3, 7.8],
            if game.heists.gold(wagon) {
                [0.5, 0.4, 0.17]
            } else {
                [0.18, 0.35, 0.45]
            },
            1,
            0.,
        );
        let name = format!("wagon_{wagon}");
        let count = write_mesh(
            &dir.join(format!("{name}.bin")),
            transformed(&mesh, p, a, [1.; 3]),
        )?;
        writeln!(manifest, "{name}\tTrain/Wagon {wagon}\t{count}")?;
    }
    manifest.flush()?;
    println!(
        "Exported all 256 LOD0 chunks, {} building interiors and sewer",
        world.buildings.iter().filter(|b| b.interior).count()
    );
    Ok(())
}

fn transformed(mesh: &Mesh, p: engine_core::Vec3, angle: f32, tint: [f32; 3]) -> Mesh {
    let mut output = Mesh::default();
    for vertex in &mesh.vertices {
        let mut v = *vertex;
        let [x, y, z] = v.position;
        v.position = [
            p.x + x * angle.cos() + z * angle.sin(),
            p.y + y,
            p.z - x * angle.sin() + z * angle.cos(),
        ];
        for (value, t) in v.color[..3].iter_mut().zip(tint) {
            *value *= t;
        }
        output.vertices.push(v);
    }
    output
}

fn wagon_chassis(mesh: &mut Mesh) {
    let signed = |v: f32| v.signum() * v.abs().powf(0.35);
    for ring in 0..4 {
        for slice in 0..8 {
            let positions = [
                (ring, slice),
                (ring, slice + 1),
                (ring + 1, slice + 1),
                (ring + 1, slice),
            ]
            .map(|(i, j)| {
                let lat = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 4.;
                let lon = std::f32::consts::TAU * j as f32 / 8.;
                [
                    signed(lat.cos()) * signed(lon.cos()) * 1.75,
                    signed(lat.sin()) * 0.7,
                    signed(lat.cos()) * signed(lon.sin()) * 4.25,
                ]
            });
            mesh.quad(positions, [0.12, 0.16, 0.2], 1, 0.);
        }
    }
}
