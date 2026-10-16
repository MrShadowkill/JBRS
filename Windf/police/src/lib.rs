use engine_core::Vec3;
pub struct Patrol {
    pub pos: Vec3,
    pub heading: f32,
    pub waypoint: Vec3,
    pub replan: f32,
    pub animation: f32,
}
pub fn tick(n: &mut Patrol, target: Vec3, wanted: u32, dt: f32) -> Vec3 {
    if wanted > 0 {
        let dx = target.x - n.pos.x;
        let dz = target.z - n.pos.z;
        let d = (dx * dx + dz * dz).sqrt();
        n.heading = dx.atan2(dz);
        if d > 1. {
            return Vec3::new(n.pos.x + dx / d * 7. * dt, 1., n.pos.z + dz / d * 7. * dt);
        }
    }
    n.pos
}
