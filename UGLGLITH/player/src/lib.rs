use engine_core::{Team, Vec3};
pub struct Player {
    pub pos: Vec3,
    pub team: Team,
    pub health: f32,
    pub vy: f32,
    pub heading: f32,
    pub animation: f32,
}
impl Player {
    pub fn new(team: Team) -> Self {
        Self {
            pos: match team {
                Team::Prisoner => Vec3::new(-100., 1., -160.),
                Team::Police => Vec3::new(0., 1., -140.),
                Team::Criminal => Vec3::new(-95., 1., -80.),
            },
            team,
            health: 100.,
            vy: 0.,
            heading: 0.,
            animation: 0.,
        }
    }
    pub fn escaped(&mut self) -> bool {
        if self.team == Team::Prisoner
            && self.pos.y > 0.2
            && (self.pos.z > -96. || self.pos.x < -175. || self.pos.x > -25.)
        {
            self.team = Team::Criminal;
            true
        } else {
            false
        }
    }
}
