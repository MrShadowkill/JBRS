use super::*;

pub fn place_0(lib: &mut Library, x: f32, z: f32) {
    lib.add(
        "Westhaven",
        x,
        z,
        [30., if x == -620. { 14. } else { 8. }, 30.],
        Style::Brick,
        x == -620.,
    );
}
