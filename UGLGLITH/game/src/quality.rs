#[derive(Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub distance: f32,
    pub detail: f32,
    pub segments: usize,
    pub grain: f32,
}
pub const PRESETS: [Preset; 4] = [
    Preset {
        name: "LOW",
        distance: 180.,
        detail: 35.,
        segments: 6,
        grain: 0.,
    },
    Preset {
        name: "MEDIUM",
        distance: 320.,
        detail: 65.,
        segments: 10,
        grain: 0.025,
    },
    Preset {
        name: "HIGH",
        distance: 450.,
        detail: 95.,
        segments: 16,
        grain: 0.05,
    },
    Preset {
        name: "ULTRA",
        distance: 600.,
        detail: 130.,
        segments: 24,
        grain: 0.075,
    },
];
