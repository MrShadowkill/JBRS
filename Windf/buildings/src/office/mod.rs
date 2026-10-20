use super::*;

pub fn place_0(lib: &mut Library, x: f32, z: f32, height: f32, ix: usize) {
    // Preserve the city blocks and streets while giving the residential
    // towers substantially more usable floors and slightly wider floor plans.
    let tower_height = (height * 2.2).min(210.);
    lib.add(
        "Office",
        x,
        z,
        [
            34. + (ix % 3) as f32 * 2.,
            (tower_height / 4.).floor() * 4.,
            34.,
        ],
        if ix.is_multiple_of(3) {
            Style::Brick
        } else {
            Style::Glass
        },
        true,
    );
}
