use super::*;

pub fn place_0(lib: &mut Library, x: f32, z: f32, height: f32, ix: usize) {
    lib.add(
        "Office",
        x,
        z,
        [28., height, 28.],
        if ix.is_multiple_of(3) {
            Style::Brick
        } else {
            Style::Glass
        },
        true,
    );
}
