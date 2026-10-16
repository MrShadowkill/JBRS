use engine_core::Mesh;
pub const LIMIT: f32 = 1024.;
pub const CHUNK: f32 = 128.;
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
pub fn industrial_height(x: f32, z: f32) -> f32 {
    18. * smooth(420., 620., x)
        * (1. - smooth(780., 980., x))
        * smooth(-260., -60., z)
        * (1. - smooth(300., 500., z))
}
pub fn height(x: f32, z: f32) -> f32 {
    let coast = (x / 1000.).powi(4) + (z / 1040.).powi(4);
    if coast > 1.05 {
        return -8. - (coast - 1.05) * 8.;
    }
    let mountain =
        smooth(390., 640., -z) * (1. - smooth(850., 990., -z)) * (1. - smooth(380., 550., x));
    let peaks = 50. + 60. * (x * 0.007).sin().powi(2) + 35. * (z * 0.011 + x * 0.004).cos().powi(2);
    let airport_flat = smooth(510., 550., x)
        * (1. - smooth(840., 880., x))
        * smooth(400., 440., z)
        * (1. - smooth(840., 880., z));
    let hills = smooth(500., 760., z) * 12. * (x * 0.009).sin().powi(2) * (1. - airport_flat);
    let beach = -5. * smooth(0.82, 1.05, coast);
    let lake = ((x + 540.) / 115.).powi(2) + ((z - 430.) / 90.).powi(2);
    if lake < 1. {
        return -4. * (1. - lake).min(0.8);
    }
    let pond = ((x + 230.) / 75.).powi(2) + ((z - 650.) / 65.).powi(2);
    if pond < 1. {
        return -3. * (1. - pond).min(0.9);
    }
    // Broad river along x=-440; every east/west arterial crosses on a bridge.
    let river = (x + 440.).abs();
    let river_depth = if (-500. ..700.).contains(&z) {
        -4. * (1. - smooth(10., 24., river))
    } else {
        0.
    };
    let inland = smooth(500., 650., x.abs().max(z.abs()));
    let rolling =
        inland * (1. - airport_flat) * 5. * (x * 0.017).sin().powi(2) * (z * 0.014).cos().powi(2);
    let industry = industrial_height(x, z);
    let meadow = smooth(340., 480., z)
        * (1. - smooth(620., 740., z))
        * (1. - smooth(-410., -360., x))
        * (1. - smooth(-750., -850., x));
    let small_lake = ((x - 580.) / 65.).powi(2) + ((z - 470.) / 48.).powi(2);
    if small_lake < 1. {
        return -3. * (1. - small_lake).min(0.9);
    }
    let stream_x = -610. + 24. * (z * 0.012).sin();
    let stream = if (280. ..740.).contains(&z) {
        -3.5 * (1. - smooth(6., 18., (x - stream_x).abs()))
    } else {
        0.
    };
    let water = (-river_depth / 4.).max(-stream / 3.5);
    let museum_distance = ((x + 200.).powi(2) + (z + 630.).powi(2)).sqrt();
    let museum_blend = smooth(220., 280., museum_distance);
    ((mountain * peaks + hills + rolling + beach + industry + meadow * 7.) * (1. - water)
        + river_depth
        + stream)
        * museum_blend
        + 88. * (1. - smooth(35., 220., museum_distance))
}
pub fn material(x: f32, z: f32, h: f32) -> (u8, [f32; 3]) {
    if h < -0.2 {
        (8, [0.25, 0.47, 0.56])
    } else if x > 500. && z < -250. {
        (5, [0.85, 0.72, 0.46])
    } else if h > 25. {
        (7, [0.62, 0.65, 0.64])
    } else if x.abs() > 820. || z.abs() > 820. {
        (5, [0.87, 0.78, 0.55])
    } else {
        (6, [0.38, 0.55, 0.32])
    }
}
pub fn mesh(key: (i32, i32), lod: u8) -> Mesh {
    mesh_reusing(key, lod, Vec::new())
}
pub fn mesh_reusing(key: (i32, i32), lod: u8, mut vertices: Vec<engine_core::Vertex>) -> Mesh {
    vertices.clear();
    let mut m = Mesh { vertices };
    let step = if lod == 0 { 8. } else { 16. };
    let n = (CHUNK / step) as i32;
    let x0 = key.0 as f32 * CHUNK;
    let z0 = key.1 as f32 * CHUNK;
    for ix in 0..n {
        for iz in 0..n {
            let x = x0 + ix as f32 * step;
            let z = z0 + iz as f32 * step;
            let p = [
                [x, height(x, z), z],
                [x + step, height(x + step, z), z],
                [x + step, height(x + step, z + step), z + step],
                [x, height(x, z + step), z + step],
            ];
            let (mat, c) = material(x, z, height(x, z));
            m.quad(p, c, mat, 0.);
            // Clip water to each terrain triangle: the shoreline follows slopes instead of square tiles.
            for indices in [[0, 1, 2], [0, 2, 3]] {
                let input = indices.map(|i| p[i]);
                let mut water = Vec::new();
                for i in 0..3 {
                    let a = input[i];
                    let b = input[(i + 1) % 3];
                    let wet = a[1] < -0.2;
                    let next = b[1] < -0.2;
                    if wet {
                        water.push([a[0], -0.2, a[2]]);
                    }
                    if wet != next {
                        let t = (-0.2 - a[1]) / (b[1] - a[1]);
                        water.push([a[0] + (b[0] - a[0]) * t, -0.2, a[2] + (b[2] - a[2]) * t]);
                    }
                }
                for i in 1..water.len().saturating_sub(1) {
                    m.triangle(
                        [water[0], water[i], water[i + 1]],
                        [0.22, 0.46, 0.61],
                        8,
                        0.1,
                    );
                }
            }
        }
    }
    m
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn industrial_highlands_residential_lowlands_and_new_water_bodies() {
        assert_eq!(height(210., 240.), 0.);
        assert_eq!(height(-220., 210.), 0.);
        assert!(height(640., 220.) > 15.);
        assert!(height(580., 470.) < -2.);
        let z = 600.;
        let x = -610. + 24. * (z * 0.012_f32).sin();
        assert!(height(x, z) < -0.2);
    }
    #[test]
    fn central_flat_and_island() {
        assert_eq!(height(-100., -160.), 0.);
        assert_eq!(height(320., 35.), 0.);
        assert!(height(-200., -630.) > 75.);
        assert!(height(1024., 1024.) < 0.);
    }
}
