use engine_core::Vec3;
pub struct Fugitive {
    pub pos: Vec3,
    pub health: f32,
    pub respawn: f32,
    pub heading: f32,
    pub animation: f32,
    pub home: Vec3,
}
impl Fugitive {
    pub fn new(x: f32, z: f32) -> Self {
        Self {
            pos: Vec3::new(x, 1., z),
            health: 100.,
            respawn: 0.,
            heading: 0.,
            animation: 0.,
            home: Vec3::new(x, 1., z),
        }
    }
    pub fn active(&self) -> bool {
        self.respawn <= 0.
    }
    pub fn arrest(&mut self) -> bool {
        if !self.active() {
            return false;
        }
        self.respawn = 60.;
        self.health = 100.;
        true
    }
    pub fn tick(&mut self, dt: f32) {
        self.respawn = (self.respawn - dt).max(0.);
    }
}
