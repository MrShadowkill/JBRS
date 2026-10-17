use engine_core::Vec3;
use world::World;
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Role {
    Resident,
    Vendor,
    Robber,
    Prisoner,
    Police,
    Swat,
}
pub struct Npc {
    pub pos: Vec3,
    pub home: Vec3,
    pub heading: f32,
    pub role: Role,
    pub phase: f32,
    pub motion: f32,
    pub cooldown: f32,
    pub waypoint: Vec3,
    pub plan: f32,
    pub health: f32,
    pub disabled: f32,
    pub last_seen: Option<Vec3>,
    pub memory: f32,
    pub aboard: bool,
    pub returning: bool,
}
impl Npc {
    pub fn new(role: Role, x: f32, z: f32) -> Self {
        let p = Vec3::new(x, 1., z);
        Self {
            pos: p,
            home: p,
            heading: 0.,
            role,
            phase: 0.,
            motion: 0.,
            cooldown: 0.,
            waypoint: p,
            plan: ((x * 0.137 + z * 0.071).abs() % 1.5) + 0.1,
            health: 100.,
            disabled: 0.,
            last_seen: None,
            memory: 0.,
            aboard: false,
            returning: false,
        }
    }
    pub fn color(&self) -> [f32; 3] {
        match self.role {
            Role::Resident => match (self.home.x as i32 / 50).rem_euclid(4) {
                0 => [0.32, 0.62, 0.46],
                1 => [0.4, 0.45, 0.68],
                2 => [0.65, 0.48, 0.29],
                _ => [0.55, 0.58, 0.6],
            },
            Role::Vendor => [0.86, 0.83, 0.7],
            Role::Robber => [0.6, 0.12, 0.18],
            Role::Prisoner => [0.92, 0.4, 0.1],
            Role::Police => [0.12, 0.25, 0.48],
            Role::Swat => [0.12, 0.16, 0.18],
        }
    }
    pub fn officer(&self) -> bool {
        matches!(self.role, Role::Police | Role::Swat)
    }
    pub fn step(&mut self, world: &World, goal: Vec3, speed: f32, dt: f32) {
        self.cooldown = (self.cooldown - dt).max(0.);
        self.disabled = (self.disabled - dt).max(0.);
        if self.disabled > 0. {
            return;
        }
        if self.health <= 0. {
            self.disabled = 45.;
            self.health = 100.;
            self.pos = self.home;
            return;
        }
        self.plan -= dt;
        if self.plan <= 0. {
            self.waypoint = world.next_step(self.pos, goal, 5.);
            self.plan = (self.pos.distance(self.waypoint) / speed.max(0.1)).clamp(0.15, 0.6);
        }
        let dx = self.waypoint.x - self.pos.x;
        let dz = self.waypoint.z - self.pos.z;
        let len = (dx * dx + dz * dz).sqrt().max(0.001);
        let d = (speed * dt).min(len);
        let mut p = Vec3::new(
            self.pos.x + dx / len * d,
            self.pos.y,
            self.pos.z + dz / len * d,
        );
        p.y = world.walking_surface(p) + 1.;
        if !world.blocked(p, 0.45) && (p.y - self.pos.y).abs() < 0.6 && self.pos.distance(p) > 0.001
        {
            let target = dx.atan2(dz);
            let delta = (target - self.heading)
                .sin()
                .atan2((target - self.heading).cos());
            self.heading += delta * (dt * 10.).min(1.);
            self.pos = p;
            self.phase += dt * speed / 6.;
            self.motion += (1. - self.motion) * (dt * 10.).min(1.);
        } else {
            self.phase += dt * 0.3;
            self.motion += (0. - self.motion) * (dt * 10.).min(1.);
        }
    }
}
pub fn population() -> Vec<Npc> {
    let mut out = Vec::new();
    for (role, x, z) in [
        (Role::Prisoner, -100., -165.),
        (Role::Prisoner, -95., -185.),
        (Role::Prisoner, -145., -190.),
        (Role::Prisoner, -145., -170.),
        (Role::Police, -100., -145.),
        (Role::Police, -50., -153.),
        (Role::Police, 0., -150.),
        (Role::Police, 18., -150.),
        (Role::Resident, 100., 15.),
        (Role::Resident, 210., 15.),
        (Role::Resident, 260., 155.),
        (Role::Resident, 350., 155.),
        (Role::Resident, -90., 185.),
        (Role::Resident, -220., 185.),
        (Role::Robber, 120., 210.),
        (Role::Robber, 290., 15.),
        (Role::Robber, -250., 100.),
        (Role::Police, 440., 15.),
    ] {
        out.push(Npc::new(role, x, z));
    }
    for (x, z) in [
        (110., 246.),
        (210., 246.),
        (-650., 46.),
        (60., 246.),
        (320., 43.),
        (-290., 108.),
        (-335., 320.),
    ] {
        out.push(Npc::new(Role::Vendor, x, z));
    }
    for (x, z) in [(450., 386.), (700., 70.), (-590., -210.)] {
        out.push(Npc::new(Role::Police, x, z));
    }
    for (x, z) in [(378., 250.), (382., 252.)] {
        let mut n = Npc::new(Role::Police, x, z);
        n.pos.y = 9.;
        n.home.y = 9.;
        out.push(n);
    }
    for (i, n) in out.iter_mut().enumerate() {
        n.plan = (i % 30) as f32 * 0.02;
    }
    out
}

pub fn sidewalk_goal(world: &World, home: Vec3, clock: f32, id: usize) -> Vec3 {
    let Some(road) = world
        .roads
        .iter()
        .min_by(|a, b| a.distance(home).0.total_cmp(&b.distance(home).0))
    else {
        return home;
    };
    let nearest = road
        .points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.distance(home).total_cmp(&b.distance(home)))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let phase = ((clock / 12.) as usize + id) % 8;
    let offset = if phase < 4 { phase } else { 7 - phase };
    let i = (nearest + offset).min(road.points.len() - 1);
    let q = road.points[i];
    let (nx, nz) = road.normal(i);
    let side = if id.is_multiple_of(2) { 1. } else { -1. };
    Vec3::new(
        q.x + nx * (road.width / 2. + 1.) * side,
        q.y + 1.,
        q.z + nz * (road.width / 2. + 1.) * side,
    )
}
