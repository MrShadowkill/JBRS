//! Embedded meshes authored offline; no asset parsing or generation in the render loop.
use engine_core::{Mesh, Vertex};
const DARK: [f32; 3] = [0.055, 0.065, 0.075];
const METAL: [f32; 3] = [0.56, 0.6, 0.63];
pub struct Model {
    pub paint: Mesh,
    pub trim: Mesh,
}
const ASSETS: [[&[u8]; 2]; engine_core::VEHICLE_SLOTS] = [
    [
        include_bytes!("../assets/vesper-low.jbv"),
        include_bytes!("../assets/vesper-high.jbv"),
    ],
    [
        include_bytes!("../assets/mica-low.jbv"),
        include_bytes!("../assets/mica-high.jbv"),
    ],
    [
        include_bytes!("../assets/sentinel-low.jbv"),
        include_bytes!("../assets/sentinel-high.jbv"),
    ],
    [
        include_bytes!("../assets/voltura-low.jbv"),
        include_bytes!("../assets/voltura-high.jbv"),
    ],
    [
        include_bytes!("../assets/ridgeback-low.jbv"),
        include_bytes!("../assets/ridgeback-high.jbv"),
    ],
    [
        include_bytes!("../assets/kestrel-low.jbv"),
        include_bytes!("../assets/kestrel-high.jbv"),
    ],
    [
        include_bytes!("../assets/aether-low.jbv"),
        include_bytes!("../assets/aether-high.jbv"),
    ],
    [
        include_bytes!("../assets/akari-low.jbv"),
        include_bytes!("../assets/akari-high.jbv"),
    ],
];

pub fn model(kind: usize, detail: bool) -> Result<Model, String> {
    let assets = ASSETS
        .get(kind)
        .ok_or_else(|| format!("Unknown vehicle model {kind}"))?;
    decode(assets[usize::from(detail)]).map_err(|e| format!("Vehicle model {kind}: {e}"))
}

pub fn catalog() -> Result<&'static [[Model; 2]], String> {
    static MODELS: std::sync::OnceLock<Result<Vec<[Model; 2]>, String>> =
        std::sync::OnceLock::new();
    match MODELS.get_or_init(|| {
        (0..ASSETS.len())
            .map(|k| Ok([model(k, false)?, model(k, true)?]))
            .collect()
    }) {
        Ok(models) => Ok(models),
        Err(error) => Err(error.clone()),
    }
}

fn decode(bytes: &[u8]) -> Result<Model, String> {
    if bytes.len() < 8 || &bytes[..4] != b"JBV1" {
        return Err("Invalid JBV header".into());
    }
    let count = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    if count == 0 || count > 8000 || bytes.len() != 8 + count * 3 * 28 {
        return Err("Invalid triangle count or truncated mesh".into());
    }
    let mut out = Model {
        paint: Mesh::default(),
        trim: Mesh::default(),
    };
    for triangle in bytes[8..].chunks_exact(84) {
        let mut material = None;
        for record in triangle.chunks_exact(28) {
            let mut values = [0f32; 7];
            for (v, b) in values.iter_mut().zip(record.chunks_exact(4)) {
                *v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
            }
            if values.iter().any(|v| !v.is_finite())
                || values[..3].iter().any(|v| v.abs() > 4.)
                || values[3..6].iter().any(|v| *v < 0. || *v > 1.)
                || !matches!(values[6], 0. | 1.)
            {
                return Err("Invalid vertex data".into());
            }
            let painted = values[6] == 1.;
            if material.is_some_and(|m| m != painted) {
                return Err("Mixed triangle materials".into());
            }
            material = Some(painted);
            let mesh = if painted {
                &mut out.paint
            } else {
                &mut out.trim
            };
            mesh.vertices.push(Vertex {
                position: [values[0], values[1], values[2]],
                color: [values[3], values[4], values[5], 0.],
                uv: [0.; 2],
            });
        }
    }
    Ok(out)
}

pub(crate) fn wheel(m: &mut Mesh, p: [f32; 3], side: f32, n: usize, detail: bool) {
    let point = |x: f32, r: f32, a: f32| [p[0] + side * x, p[1] + r * a.cos(), p[2] + r * a.sin()];
    for i in 0..n {
        let a = i as f32 * std::f32::consts::TAU / n as f32;
        let b = (i + 1) as f32 * std::f32::consts::TAU / n as f32;
        for (x0, r0, x1, r1) in [
            (-0.15, 0.29, -0.1, 0.35),
            (-0.1, 0.35, 0.15, 0.35),
            (0.15, 0.35, 0.19, 0.29),
            (0.19, 0.29, 0.19, 0.23),
        ] {
            m.quad(
                [
                    point(x0, r0, a),
                    point(x1, r1, a),
                    point(x1, r1, b),
                    point(x0, r0, b),
                ],
                DARK,
                1,
                0.,
            );
        }
        m.triangle(
            [
                point(0.195, 0., a),
                point(0.195, 0.23, a),
                point(0.195, 0.23, b),
            ],
            if detail { [0.2, 0.22, 0.24] } else { METAL },
            1,
            0.,
        );
        if detail {
            m.quad(
                [
                    point(0.2, 0.23, a),
                    point(0.2, 0.255, a),
                    point(0.2, 0.255, b),
                    point(0.2, 0.23, b),
                ],
                METAL,
                1,
                0.,
            );
        }
    }
    if detail {
        for i in 0..5 {
            let a = i as f32 * std::f32::consts::TAU / 5.;
            m.quad(
                [
                    point(0.21, 0.055, a - 0.25),
                    point(0.21, 0.23, a - 0.1),
                    point(0.21, 0.23, a + 0.1),
                    point(0.21, 0.055, a + 0.25),
                ],
                METAL,
                1,
                0.,
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_catalog_model_has_bounded_geometry_and_real_lod() {
        for kind in 0..ASSETS.len() {
            let near = model(kind, true).unwrap();
            let far = model(kind, false).unwrap();
            let count = |m: &Model| (m.paint.vertices.len() + m.trim.vertices.len()) / 3;
            assert!(count(&near) < 6000);
            assert!(count(&far) * 2 < count(&near));
            assert!(!near.paint.vertices.is_empty() && !near.trim.vertices.is_empty());
            for m in [&near, &far] {
                for v in m.paint.vertices.iter().chain(&m.trim.vertices) {
                    assert!(
                        v.position[0].abs() < 1.5
                            && v.position[1] >= -0.9
                            && v.position[1] < 2.1
                            && v.position[2].abs() < 2.8
                    );
                }
            }
        }
    }
    #[test]
    fn imported_headlights_face_the_same_direction_as_vehicle_physics() {
        let model = model(7, true).unwrap();
        let lights: Vec<_> = model
            .trim
            .vertices
            .iter()
            .filter(|v| {
                let [r, g, b, _] = v.color;
                r > b * 1.1 && g > b * 1.05 && b > 0.35
            })
            .collect();
        assert!(!lights.is_empty());
        assert!(lights.iter().all(|v| v.position[2] > 1.5));
    }

    #[test]
    fn imported_body_skirts_are_paintable_but_wheel_centers_are_not() {
        for detail in [false, true] {
            let model = model(7, detail).unwrap();
            let wheel_center = |v: &engine_core::Vertex| {
                let [x, y, z] = v.position;
                x.abs() > 0.7
                    && [1.4, -1.2]
                        .iter()
                        .any(|center| (z - center).powi(2) + (y + 0.3).powi(2) < 0.18f32.powi(2))
            };
            assert!(model.trim.vertices.iter().any(wheel_center));
            assert!(!model.paint.vertices.iter().any(wheel_center));
            assert!(model
                .paint
                .vertices
                .iter()
                .any(|v| v.position[0].abs() > 0.8
                    && v.position[1] < -0.15
                    && v.position[2].abs() < 0.8));
        }
    }

    #[test]
    fn malformed_assets_return_meaningful_errors() {
        assert!(decode(b"JBV1").is_err());
        let mut bytes = ASSETS[7][1].to_vec();
        bytes.pop();
        assert!(decode(&bytes).is_err());
        let mut bytes = ASSETS[7][1].to_vec();
        bytes[8..12].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode(&bytes).is_err());
        assert!(model(ASSETS.len(), true).is_err());
    }
    #[test]
    fn catalog_meshes_are_distinct_and_cached() {
        let a = catalog().unwrap();
        let b = catalog().unwrap();
        assert!(std::ptr::eq(a, b));
        for (i, asset) in ASSETS.iter().enumerate() {
            for other in &ASSETS[i + 1..] {
                assert_ne!(asset[1], other[1]);
            }
        }
    }
}
