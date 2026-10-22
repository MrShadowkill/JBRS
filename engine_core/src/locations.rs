//! Shared world coordinates; model-local Z=-8 is the east-facing entrance.
pub mod donut {
    use crate::Vec3;
    pub const SCALE: f32 = 1.5;
    pub const CENTER: Vec3 = Vec3 {
        x: 193.,
        y: 0.,
        z: -80.,
    };
    pub const DOOR: Vec3 = Vec3 {
        x: 205.,
        y: 1.,
        z: -80.,
    };
    pub const REGISTER: Vec3 = Vec3 {
        x: 185.35,
        y: 2.325,
        z: -72.5,
    };
    pub const REGISTER_REACH: Vec3 = Vec3 {
        x: 186.625,
        y: 1.5,
        z: -72.5,
    };
    pub const ROAD_X: f32 = 226.;
    pub fn point(local: [f32; 3]) -> Vec3 {
        Vec3::new(
            CENTER.x - local[2] * SCALE,
            local[1] * SCALE,
            CENTER.z + local[0] * SCALE,
        )
    }
    pub fn hinge(side: f32) -> Vec3 {
        point([side * 2.05, 0., -7.95])
    }
}
