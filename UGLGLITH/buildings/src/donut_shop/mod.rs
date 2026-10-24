use super::*;
pub use engine_core::locations::donut::{CENTER, DOOR, REGISTER};
use std::sync::OnceLock;
#[allow(clippy::excessive_precision)]
mod geometry {
    include!("assets/generated.rs");
}
pub use geometry::GLASS;
pub struct Model {
    pub body: Mesh,
    pub roof: Mesh,
    pub left: Mesh,
    pub right: Mesh,
}
fn decode(bytes: &[u8]) -> Mesh {
    let vertices = bytes
        .chunks_exact(28)
        .map(|chunk| {
            let f =
                |i: usize| f32::from_le_bytes([chunk[i], chunk[i + 1], chunk[i + 2], chunk[i + 3]]);
            engine_core::Vertex {
                position: [f(0), f(4), f(8)],
                color: [f(12), f(16), f(20), f(24)],
                uv: [0.3, 0.3],
            }
        })
        .collect();
    Mesh { vertices }
}
pub fn model() -> &'static Model {
    static DATA: OnceLock<Model> = OnceLock::new();
    DATA.get_or_init(|| Model {
        body: decode(include_bytes!("assets/model.bin")),
        roof: decode(include_bytes!("assets/roof.bin")),
        left: decode(include_bytes!("assets/left.bin")),
        right: decode(include_bytes!("assets/right.bin")),
    })
}
fn placed(source: &Mesh, b: &Building, include_y: bool) -> Mesh {
    Mesh {
        vertices: source
            .vertices
            .iter()
            .map(|v| {
                let mut v = *v;
                v.position[1] += 0.06;
                v.position[0] += b.p.x;
                v.position[2] += b.p.z;
                if include_y {
                    v.position[1] += b.p.y;
                }
                v
            })
            .collect(),
    }
}
pub fn exterior(b: &Building, m: &mut Mesh) {
    m.vertices.extend(placed(&model().body, b, false).vertices);
}
pub fn roof(b: &Building) -> Mesh {
    placed(&model().roof, b, true)
}
pub fn colliders(b: &Building) -> Vec<Collider> {
    geometry::COLLISIONS
        .iter()
        .map(|(p, s)| Collider {
            p: Vec3::new(b.p.x + p[0], p[1] + 0.06, b.p.z + p[2]),
            size: Vec3::new(s[0], s[1], s[2]),
            movable: false,
            breakable: false,
        })
        .collect()
}
pub fn place_0(lib: &mut Library) {
    lib.add(
        "Donut Shop",
        CENTER.x,
        CENTER.z,
        [26.1, 17.1, 32.1],
        Style::Neon,
        true,
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imported_geometry_keeps_east_doorway_and_register_access_clear() {
        let mut lib = Library::default();
        place_0(&mut lib);
        let b = &lib.buildings[0];
        let c = b.colliders();
        assert!(b.entrance().distance(Vec3::new(DOOR.x + 2., 1., DOOR.z)) < 0.01);
        for x in 200..211 {
            assert!(!c
                .iter()
                .any(|c| c.contains(Vec3::new(x as f32, 1., DOOR.z), 0.45)));
        }
        assert!(c
            .iter()
            .any(|c| c.contains(Vec3::new(CENTER.x - 11.8, 1., CENTER.z + 7.), 0.45)));
        assert!(model().body.vertices.len() > 3000);
        assert!(!model().roof.vertices.is_empty());
    }
}
