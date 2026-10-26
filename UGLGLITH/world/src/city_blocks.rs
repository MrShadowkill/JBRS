//! Small infill parcels preserve the original gameplay buildings, IDs and entrance approaches.
use super::*;
pub(super) fn infill(roads: &[Road], lib: &mut Library, props: &[Prop]) {
    let mut placed = 0;
    for (ix, x) in [30., 85., 135., 185., 235., 285., 335., 385., 445.]
        .into_iter()
        .enumerate()
    {
        for (iz, z) in [30., 75., 125., 170., 215., 265.].into_iter().enumerate() {
            let width = 12. + ((ix + iz) % 3) as f32 * 2.;
            let depth = 14.;
            let p = Vec3::new(x, 0., z);
            let radius = (width * width + depth * depth).sqrt() / 2. + 3.;
            if roads
                .iter()
                .any(|r| r.distance(p).0 < r.width / 2. + radius)
            {
                continue;
            }
            if lib.buildings.iter().any(|b| {
                let overlap = (b.p.x - x).abs() < b.size.x / 2. + width / 2. + 4.
                    && (b.p.z - z).abs() < b.size.z / 2. + depth / 2. + 4.;
                let approach = b.interior
                    && (b.p.x - x).abs() < b.size.x / 2. + width / 2. + 3.
                    && z > b.p.z - b.size.z / 2. - depth / 2. - 12.
                    && z < b.p.z;
                overlap || approach
            }) {
                continue;
            }
            if props.iter().any(|prop| {
                (prop.p.x - x).abs() < prop.size.x / 2. + width / 2. + 2.
                    && (prop.p.z - z).abs() < prop.size.z / 2. + depth / 2. + 2.
            }) {
                continue;
            }
            lib.add(
                "City Annex",
                x,
                z,
                [width, 12. + ((ix * 3 + iz) % 4) as f32 * 4., depth],
                buildings::Style::Brick,
                false,
            );
            if let Some(b) = lib.buildings.last_mut() {
                b.p.y = terrain::height(x, z).max(0.);
            }
            placed += 1;
            if placed == 22 {
                return;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn infill_is_bounded_and_keeps_original_entrances_and_roads_clear() {
        let w = World::new();
        let annexes: Vec<_> = w
            .buildings
            .iter()
            .filter(|b| b.name == "City Annex")
            .collect();
        assert!(!annexes.is_empty());
        assert!(annexes.len() <= 22);
        w.audit().unwrap();
        for a in annexes {
            for b in w.buildings.iter().filter(|b| b.id != a.id) {
                assert!(
                    (a.p.x - b.p.x).abs() >= (a.size.x + b.size.x) / 2. + 3.
                        || (a.p.z - b.p.z).abs() >= (a.size.z + b.size.z) / 2. + 3.
                );
            }
        }
    }
}
