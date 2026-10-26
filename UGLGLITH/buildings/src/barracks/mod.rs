use super::*;

pub fn place_0(lib: &mut Library, x: f32) {
    lib.add(
        "Barracks",
        x,
        -560.,
        [28., 6., 24.],
        Style::Warehouse,
        false,
    );
}
