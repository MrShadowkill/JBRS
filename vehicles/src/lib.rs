pub mod body;
pub mod rail;
use engine_core::Vec3;
pub struct Spec {
    pub name: &'static str,
    pub price: u32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub grip: f32,
    pub color: [f32; 3],
}
pub const CATALOG: [Spec; engine_core::VEHICLE_SLOTS] = [
    Spec {
        name: "Vesper C240",
        price: 0,
        max_speed: 43.,
        acceleration: 12.,
        grip: 3.5,
        color: [0.8, 0.24, 0.12],
    },
    Spec {
        name: "Mica Cityline",
        price: 0,
        max_speed: 35.,
        acceleration: 14.,
        grip: 4.2,
        color: [0.22, 0.7, 0.53],
    },
    Spec {
        name: "Sentinel Patrol",
        price: 0,
        max_speed: 49.,
        acceleration: 15.,
        grip: 4.,
        color: [0.14, 0.28, 0.7],
    },
    Spec {
        name: "Voltura GT",
        price: 2500,
        max_speed: 40.,
        acceleration: 16.,
        grip: 2.8,
        color: [0.88, 0.7, 0.22],
    },
    Spec {
        name: "Ridgeback 250",
        price: 6000,
        max_speed: 38.,
        acceleration: 10.,
        grip: 3.,
        color: [0.45, 0.53, 0.58],
    },
    Spec {
        name: "Kestrel RS",
        price: 12000,
        max_speed: 60.,
        acceleration: 18.,
        grip: 2.3,
        color: [0.65, 0.18, 0.5],
    },
    Spec {
        name: "Aether V12",
        price: 24000,
        max_speed: 72.,
        acceleration: 20.,
        grip: 3.8,
        color: [0.05, 0.65, 0.92],
    },
    Spec {
        name: "Akari R7",
        price: 18000,
        max_speed: 57.,
        acceleration: 17.,
        grip: 3.7,
        color: [0.9, 0.91, 0.94],
    },
];
pub struct Car {
    pub pos: Vec3,
    pub angle: f32,
    pub speed: f32,
    /// Smoothed steering input in the range -1..=1. This is runtime-only state.
    pub steering: f32,
    /// Accumulated tire rotation in radians, used by the wheel renderer.
    pub wheel_rotation: f32,
    pub vertical_speed: f32,
    pub tilt: [f32; 2],
    pub kind: usize,
    pub engine_level: u32,
    pub brake_level: u32,
    pub suspension_level: u32,
    pub damage: f32,
    pub color: [f32; 3],
}
impl Car {
    pub fn new(kind: usize, pos: Vec3) -> Self {
        Self {
            pos,
            angle: 0.,
            speed: 0.,
            steering: 0.,
            wheel_rotation: 0.,
            vertical_speed: 0.,
            tilt: [0.; 2],
            kind,
            engine_level: 0,
            brake_level: 0,
            suspension_level: 0,
            damage: 0.,
            color: CATALOG[kind].color,
        }
    }
}
