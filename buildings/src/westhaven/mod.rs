use super::*;

pub fn place_0(lib: &mut Library, x: f32, z: f32) {
    lib.add(
        "Westhaven",
        x,
        z,
        [32., if x == -620. { 42. } else { 8. }, 32.],
        Style::Brick,
        x == -620.,
    );
}
