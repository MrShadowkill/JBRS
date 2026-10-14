pub mod body;
use engine_core::Vec3;
pub struct Spec {
    pub name: &'static str,
    pub price: u32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub grip: f32,
    pub color: [f32; 3],
}
pub const CATALOG: [Spec; 7] = [
    Spec {
        name: "Starter Coupe",
        price: 0,
        max_speed: 43.,
        acceleration: 12.,
        grip: 3.5,
        color: [0.8, 0.24, 0.12],
    },
    Spec {
        name: "Starter Hatchback",
        price: 0,
        max_speed: 35.,
        acceleration: 14.,
        grip: 4.2,
        color: [0.22, 0.7, 0.53],
    },
    Spec {
        name: "Police Cruiser",
        price: 0,
        max_speed: 49.,
        acceleration: 15.,
        grip: 4.,
        color: [0.14, 0.28, 0.7],
    },
    Spec {
        name: "Electric GT",
        price: 2500,
        max_speed: 40.,
        acceleration: 16.,
        grip: 2.8,
        color: [0.88, 0.7, 0.22],
    },
    Spec {
        name: "Pickup Truck",
        price: 6000,
        max_speed: 38.,
        acceleration: 10.,
        grip: 3.,
        color: [0.45, 0.53, 0.58],
    },
    Spec {
        name: "Track Sport",
        price: 12000,
        max_speed: 60.,
        acceleration: 18.,
        grip: 2.3,
        color: [0.65, 0.18, 0.5],
    },
    Spec {
        name: "Apex Hypercar",
        price: 24000,
        max_speed: 72.,
        acceleration: 20.,
        grip: 3.8,
        color: [0.05, 0.65, 0.92],
    },
];
pub struct Car {
    pub pos: Vec3,
    pub angle: f32,
    pub speed: f32,
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
            kind,
            engine_level: 0,
            brake_level: 0,
            suspension_level: 0,
            damage: 0.,
            color: CATALOG[kind].color,
        }
    }
}
