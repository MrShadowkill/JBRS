//! Bounded, allocation-free chaos simulation. GPU assets are created once at startup.
#[path = "absurd.rs"]
mod absurd;
use crate::npcs::Npc;
pub use absurd::Interaction;
use engine_core::{Mesh, Vec3};
use renderer_gl21::{GpuMesh, Renderer, Texture};
use vehicles::Car;
use world::World;

pub const CAPACITY: usize = 512;
pub const BRIDGE_CAPACITY: usize = 128;
const TITANS: std::ops::Range<usize> = 0..15;
const STALKERS: std::ops::Range<usize> = 15..47;
const COPS: std::ops::Range<usize> = 47..111;
const PEASANTS: std::ops::Range<usize> = 111..175;
const LOGS: [&str; 4] = [
    "ERROR: physics.rs:404",
    "LAG: 999ms",
    "missing texture",
    "OpenGL crash?",
];

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Entity {
    #[default]
    Empty,
    PiskaZaGuboy,
    GachiTitan,
    PinkCopOverflow,
    Peasant18thCentury,
}

pub struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 32) as u32
    }
    fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
    fn unit(&mut self) -> f32 {
        (self.next() >> 8) as f32 / 16_777_216.
    }
    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}
fn dot(a: Vec3, b: Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}
fn direction(a: Vec3) -> Vec3 {
    let d = dot(a, a).sqrt();
    if d > 0.0001 && d.is_finite() {
        Vec3::new(a.x / d, a.y / d, a.z / d)
    } else {
        Vec3::default()
    }
}
fn finite(v: f32, limit: f32) -> f32 {
    if v.is_finite() {
        v.clamp(-limit, limit)
    } else {
        0.
    }
}
fn integrate(p: &mut Vec3, v: &mut Vec3, dt: f32) {
    v.y -= 18. * dt;
    v.x = finite(v.x, 9999.);
    v.y = finite(v.y, 9999.);
    v.z = finite(v.z, 9999.);
    p.x = finite(p.x + v.x * dt, 100_000.);
    p.y = finite(p.y + v.y * dt, 100_000.);
    p.z = finite(p.z + v.z * dt, 100_000.);
}
fn overlaps(a: Vec3, ah: Vec3, b: Vec3, bh: Vec3) -> bool {
    (a.x - b.x).abs() <= ah.x + bh.x
        && (a.y - b.y).abs() <= ah.y + bh.y
        && (a.z - b.z).abs() <= ah.z + bh.z
}

pub struct SchizoSpawner {
    pub kind: [Entity; CAPACITY],
    pub position: [Vec3; CAPACITY],
    pub velocity: [Vec3; CAPACITY],
    drift: [Vec3; CAPACITY],
    pub scale: [f32; CAPACITY],
    pub mass: [f32; CAPACITY],
    pub yaw: [f32; CAPACITY],
    pub phase: [f32; CAPACITY],
    pub frozen: [bool; CAPACITY],
    pub infected: [bool; CAPACITY],
    center: Vec3,
    initialized: bool,
    peasant_wait: f32,
    cop_wait: f32,
    recycle: usize,
    circle: f32,
}
impl SchizoSpawner {
    fn new() -> Self {
        Self {
            kind: [Entity::Empty; CAPACITY],
            position: [Vec3::default(); CAPACITY],
            velocity: [Vec3::default(); CAPACITY],
            drift: [Vec3::default(); CAPACITY],
            scale: [1.; CAPACITY],
            mass: [80.; CAPACITY],
            yaw: [0.; CAPACITY],
            phase: [0.; CAPACITY],
            frozen: [false; CAPACITY],
            infected: [false; CAPACITY],
            center: Vec3::default(),
            initialized: false,
            peasant_wait: 10.,
            cop_wait: 1.,
            recycle: PEASANTS.start,
            circle: 0.,
        }
    }
    fn activate(&mut self, i: usize, kind: Entity, p: Vec3) {
        self.kind[i] = kind;
        self.position[i] = p;
        self.velocity[i] = Vec3::default();
        self.drift[i] = Vec3::default();
        self.scale[i] = if kind == Entity::GachiTitan { 2.5 } else { 1. };
        self.mass[i] = if kind == Entity::GachiTitan {
            f32::MAX
        } else {
            80.
        };
        self.yaw[i] = 0.;
        self.phase[i] = 0.;
        self.frozen[i] = false;
        self.infected[i] = false;
    }
    fn initialize(&mut self, world: &World, player: Vec3) {
        self.initialized = true;
        self.center = player;
        for i in TITANS.chain(STALKERS.start..STALKERS.start + 8) {
            let a = i as f32 * 2.399_963;
            let distance = if i < TITANS.end { 22. } else { 15. };
            let mut p = Vec3::new(
                player.x + a.cos() * distance,
                0.,
                player.z + a.sin() * distance,
            );
            p.y = world.surface(p.x, p.z) + if i < TITANS.end { 2.5 } else { 1. };
            self.activate(
                i,
                if i < TITANS.end {
                    Entity::GachiTitan
                } else {
                    Entity::PiskaZaGuboy
                },
                p,
            );
        }
    }
    fn update(
        &mut self,
        world: &World,
        player: Vec3,
        eye: Vec3,
        forward: Vec3,
        rng: &mut Rng,
        dt: f32,
    ) {
        if !self.initialized {
            self.initialize(world, player);
        }
        self.peasant_wait -= dt;
        self.cop_wait -= dt;
        self.circle = (self.circle + dt * 0.9).rem_euclid(std::f32::consts::TAU);
        if self.peasant_wait <= 0. {
            self.peasant_wait += 10.;
            let i = PEASANTS
                .clone()
                .find(|&i| self.kind[i] == Entity::Empty)
                .unwrap_or(self.recycle);
            self.recycle = PEASANTS.start + (i + 1 - PEASANTS.start) % PEASANTS.len();
            self.activate(
                i,
                Entity::Peasant18thCentury,
                Vec3::new(
                    player.x + rng.range(-12., 12.),
                    100.,
                    player.z + rng.range(-12., 12.),
                ),
            );
            self.velocity[i] = Vec3::new(rng.range(-2., 2.), -1., rng.range(-2., 2.));
        }
        if self.cop_wait <= 0. {
            self.cop_wait += 1.;
            if let Some(i) = COPS.clone().find(|&i| self.kind[i] == Entity::Empty) {
                self.activate(
                    i,
                    Entity::PinkCopOverflow,
                    Vec3::new(
                        self.center.x + 10.,
                        world.surface(self.center.x + 10., self.center.z) + 1.,
                        self.center.z,
                    ),
                );
            }
        }
        for i in 0..CAPACITY {
            let kind = self.kind[i];
            if kind == Entity::Empty || self.frozen[i] {
                continue;
            }
            if kind == Entity::PiskaZaGuboy && dot(sub(self.position[i], eye), forward) >= 0. {
                continue;
            }
            match kind {
                Entity::PiskaZaGuboy | Entity::GachiTitan => {
                    let mut d = sub(player, self.position[i]);
                    d.y = 0.;
                    d = direction(d);
                    let speed = if kind == Entity::GachiTitan { 2. } else { 6. };
                    self.velocity[i].x = d.x * speed;
                    self.velocity[i].z = d.z * speed;
                    if kind == Entity::GachiTitan {
                        self.scale[i] = 2.5;
                        self.mass[i] = f32::MAX;
                    } else {
                        self.yaw[i] = (self.yaw[i] + 1000. * dt).rem_euclid(std::f32::consts::TAU);
                    }
                }
                Entity::PinkCopOverflow => {
                    let a = self.circle
                        + (i - COPS.start) as f32 * std::f32::consts::TAU / COPS.len() as f32;
                    let target = Vec3::new(
                        self.center.x + a.cos() * 10.,
                        self.position[i].y,
                        self.center.z + a.sin() * 10.,
                    );
                    let d = direction(sub(target, self.position[i]));
                    self.velocity[i].x = d.x * 7.;
                    self.velocity[i].z = d.z * 7.;
                    self.yaw[i] = (-d.x).atan2(-d.z);
                    if !self.infected[i] {
                        self.phase[i] = (self.phase[i] - dt * 9.).rem_euclid(std::f32::consts::TAU);
                    }
                }
                _ => {}
            }
            integrate(&mut self.position[i], &mut self.velocity[i], dt);
            self.position[i].x += self.drift[i].x * dt;
            self.position[i].z += self.drift[i].z * dt;
            self.drift[i].x *= 1. - dt * 2.;
            self.drift[i].z *= 1. - dt * 2.;
            let floor = world.surface(self.position[i].x, self.position[i].z) + self.scale[i];
            if self.position[i].y <= floor {
                self.position[i].y = floor;
                self.velocity[i].y = 0.;
                if kind == Entity::Peasant18thCentury {
                    self.frozen[i] = true;
                    self.velocity[i] = Vec3::default();
                }
            }
        }
        for cop in COPS {
            if self.kind[cop] == Entity::Empty {
                continue;
            }
            for p in PEASANTS {
                if self.frozen[p]
                    && overlaps(
                        self.position[cop],
                        Vec3::new(0.4, 1., 0.4),
                        self.position[p],
                        Vec3::new(0.4, 1., 0.4),
                    )
                {
                    self.infected[cop] = true;
                    self.phase[cop] = 0.;
                    break;
                }
            }
        }
    }
    pub fn blocks(&self, p: Vec3, r: f32) -> bool {
        PEASANTS.into_iter().any(|i| {
            self.frozen[i]
                && overlaps(
                    p,
                    Vec3::new(r, 0.8, r),
                    self.position[i],
                    Vec3::new(0.4, 1., 0.4),
                )
        })
    }
}

pub struct PhysicsSaboteur {
    frames: u16,
    pub car_scale: [f32; BRIDGE_CAPACITY],
    pub npc_scale: [f32; BRIDGE_CAPACITY],
    pub npc_infected: [bool; BRIDGE_CAPACITY],
    car_airborne: [bool; BRIDGE_CAPACITY],
    car_previous: [Vec3; BRIDGE_CAPACITY],
    tracked_cars: usize,
    car_position: [Vec3; BRIDGE_CAPACITY],
    car_velocity: [Vec3; BRIDGE_CAPACITY],
    car_bypass: [f32; BRIDGE_CAPACITY],
    npc_airborne: [bool; BRIDGE_CAPACITY],
    npc_position: [Vec3; BRIDGE_CAPACITY],
    npc_velocity: [Vec3; BRIDGE_CAPACITY],
}
impl PhysicsSaboteur {
    fn new(rng: &mut Rng) -> Self {
        Self {
            frames: 300 + rng.below(201) as u16,
            car_scale: [1.; BRIDGE_CAPACITY],
            npc_scale: [1.; BRIDGE_CAPACITY],
            npc_infected: [false; BRIDGE_CAPACITY],
            car_airborne: [false; BRIDGE_CAPACITY],
            car_previous: [Vec3::default(); BRIDGE_CAPACITY],
            tracked_cars: 0,
            car_position: [Vec3::default(); BRIDGE_CAPACITY],
            car_velocity: [Vec3::default(); BRIDGE_CAPACITY],
            car_bypass: [0.; BRIDGE_CAPACITY],
            npc_airborne: [false; BRIDGE_CAPACITY],
            npc_position: [Vec3::default(); BRIDGE_CAPACITY],
            npc_velocity: [Vec3::default(); BRIDGE_CAPACITY],
        }
    }
    fn launch(&mut self, i: usize, p: Vec3, v: Vec3, bypass: f32) {
        if i >= BRIDGE_CAPACITY {
            return;
        }
        if !self.car_airborne[i] {
            self.car_position[i] = p;
        }
        self.car_airborne[i] = true;
        self.car_velocity[i] = v;
        self.car_bypass[i] = bypass;
    }
    pub fn wall_active(eye: Vec3, forward: Vec3, c: &buildings::Collider) -> bool {
        // Find the surface being crossed, rather than the building's distant center.
        let nearest = Vec3::new(
            eye.x.clamp(c.p.x - c.size.x / 2., c.p.x + c.size.x / 2.),
            eye.y.clamp(c.p.y - c.size.y / 2., c.p.y + c.size.y / 2.),
            eye.z.clamp(c.p.z - c.size.z / 2., c.p.z + c.size.z / 2.),
        );
        let d = direction(sub(nearest, eye));
        dot(d, d) > 0.0001 && dot(d, forward) < 0.85
    }
    fn update(
        &mut self,
        pool: &SchizoSpawner,
        world: &World,
        cars: &mut [Car],
        npcs: &mut [Npc],
        rng: &mut Rng,
        dt: f32,
    ) {
        for (i, c) in cars
            .iter()
            .take(BRIDGE_CAPACITY)
            .enumerate()
            .skip(self.tracked_cars)
        {
            self.car_previous[i] = c.pos;
        }
        self.tracked_cars = cars.len().min(BRIDGE_CAPACITY);
        self.frames = self.frames.saturating_sub(1);
        if self.frames == 0 {
            self.frames = 300 + rng.below(201) as u16;
            let n = cars.len().min(BRIDGE_CAPACITY);
            if n > 0 {
                let i = rng.below(n as u32) as usize;
                self.launch(
                    i,
                    cars[i].pos,
                    Vec3::new(0., if rng.below(2) == 0 { 9999. } else { -9999. }, 0.),
                    2.,
                );
            }
        }
        for (i, c) in cars.iter_mut().take(BRIDGE_CAPACITY).enumerate() {
            for titan in TITANS {
                if pool.kind[titan] == Entity::GachiTitan
                    && overlaps(
                        c.pos,
                        Vec3::new(
                            self.car_scale[i],
                            0.8 * self.car_scale[i],
                            2. * self.car_scale[i],
                        ),
                        pool.position[titan],
                        Vec3::new(1., 2.5, 1.),
                    )
                {
                    self.launch(i, c.pos, Vec3::new(0., 999., 0.), 1.);
                    break;
                }
            }
            if !self.car_airborne[i] && pool.blocks(c.pos, 2. * self.car_scale[i]) {
                c.pos = self.car_previous[i];
                c.speed = 0.;
            }
            if self.car_airborne[i] {
                self.car_bypass[i] = (self.car_bypass[i] - dt).max(0.);
                integrate(&mut self.car_position[i], &mut self.car_velocity[i], dt);
                let floor = world.surface(self.car_position[i].x, self.car_position[i].z) + 0.65;
                if self.car_bypass[i] == 0. && self.car_position[i].y <= floor {
                    self.car_position[i].y = floor;
                    self.car_velocity[i] = Vec3::default();
                    self.car_airborne[i] = false;
                }
                c.pos = self.car_position[i];
                c.vertical_speed = self.car_velocity[i].y;
            }
            self.car_previous[i] = c.pos;
        }
        for (i, n) in npcs.iter_mut().take(BRIDGE_CAPACITY).enumerate() {
            if n.officer() && pool.blocks(n.pos, 0.5) {
                self.npc_infected[i] = true;
            }
            if self.npc_infected[i] {
                n.phase = 0.;
                n.motion = 0.;
            }
            if self.npc_airborne[i] {
                integrate(&mut self.npc_position[i], &mut self.npc_velocity[i], dt);
                let floor = world.walking_surface(self.npc_position[i]) + 1.;
                if self.npc_position[i].y <= floor {
                    self.npc_position[i].y = floor;
                    self.npc_airborne[i] = false;
                }
                n.pos = self.npc_position[i];
            }
        }
    }
}

pub struct RenderSaboteur {
    missing: Option<Texture>,
    actor: Option<GpuMesh>,
    pub car_corrupt: [bool; BRIDGE_CAPACITY],
    pub npc_corrupt: [bool; BRIDGE_CAPACITY],
    wait: f32,
    remaining: f32,
    blink: f32,
    alpha: f32,
    log: usize,
    xy: [f32; 2],
    ram_seconds: f32,
    ram_reset_in: f32,
    fake_gb: u64,
    ram: [u8; 48],
    ram_len: usize,
}
impl RenderSaboteur {
    pub const fn missing_pixels() -> [u8; 16 * 16 * 3] {
        let mut data = [0; 16 * 16 * 3];
        let mut y = 0;
        while y < 16 {
            let mut x = 0;
            while x < 16 {
                let p = (y * 16 + x) * 3;
                if (x / 4 + y / 4) % 2 == 0 {
                    data[p] = 255;
                    data[p + 2] = 255;
                }
                x += 1;
            }
            y += 1;
        }
        data
    }
    fn new(rng: &mut Rng) -> Self {
        let mut s = Self {
            missing: None,
            actor: None,
            car_corrupt: [false; BRIDGE_CAPACITY],
            npc_corrupt: [false; BRIDGE_CAPACITY],
            wait: rng.range(10., 20.),
            remaining: 0.,
            blink: 0.,
            alpha: 0.,
            log: 0,
            xy: [0.; 2],
            ram_seconds: 0.,
            ram_reset_in: 20.,
            fake_gb: 0,
            ram: [0; 48],
            ram_len: 0,
        };
        for i in 0..BRIDGE_CAPACITY {
            s.car_corrupt[i] = rng.below(100) < 15;
            s.npc_corrupt[i] = rng.below(100) < 15;
        }
        s.ram_reset_in = rng.range(20., 40.);
        s.write_ram();
        s
    }
    pub fn initialize(&mut self, r: &mut Renderer) {
        if self.actor.is_some() {
            return;
        }
        self.missing = Some(r.texture(16, 16, &Self::missing_pixels()));
        let mut mesh = Mesh {
            vertices: Vec::with_capacity(216),
        };
        for (p, size) in [
            ([0., 0.1, 0.], [0.7, 0.8, 0.35]),
            ([0., 0.75, 0.], [0.45, 0.45, 0.45]),
            ([-0.75, 0.32, 0.], [0.85, 0.22, 0.25]),
            ([0.75, 0.32, 0.], [0.85, 0.22, 0.25]),
            ([-0.2, -0.6, 0.], [0.28, 0.8, 0.3]),
            ([0.2, -0.6, 0.], [0.28, 0.8, 0.3]),
        ] {
            mesh.cuboid(p, size, [1.; 3], 0, 0.);
        }
        for (i, v) in mesh.vertices.iter_mut().enumerate() {
            v.color = [1., 1., 1., (i + 1) as f32];
            v.uv = v.uv.map(|v| (v - 0.005) / 0.24);
        }
        self.actor = Some(r.upload(&mesh));
        // All lazy vehicle VBO uploads happen once before the main loop.
        r.prepare_chaos_actor();
    }
    fn write_ram(&mut self) {
        let (mut value, suffix) = if self.fake_gb >= 1000 {
            (self.fake_gb / 1000, " TB RAM USED")
        } else {
            (self.fake_gb, " GB RAM USED")
        };
        let mut digits = [0u8; 20];
        let mut n = 0;
        loop {
            digits[n] = b'0' + (value % 10) as u8;
            n += 1;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        self.ram_len = 0;
        for i in (0..n).rev() {
            self.ram[self.ram_len] = digits[i];
            self.ram_len += 1;
        }
        for b in suffix.bytes() {
            self.ram[self.ram_len] = b;
            self.ram_len += 1;
        }
    }
    fn update(&mut self, rng: &mut Rng, dt: f32) {
        self.ram_reset_in -= dt;
        self.ram_seconds += dt;
        if self.ram_seconds >= 1. {
            self.ram_seconds -= 1.;
            self.fake_gb = self.fake_gb.saturating_add(1).min(999_999_000);
            self.write_ram();
        }
        if self.ram_reset_in <= 0. {
            // Instant rollback, then a fresh 1 GB/s accumulation cycle.
            self.fake_gb = 0;
            self.ram_seconds = 0.;
            self.ram_reset_in = rng.range(20., 40.);
            self.write_ram();
        }
        self.wait -= dt;
        self.remaining = (self.remaining - dt).max(0.);
        self.blink -= dt;
        if self.wait <= 0. {
            self.wait = rng.range(10., 20.);
            self.remaining = 4.;
            self.log = rng.below(4) as usize;
            self.xy = [rng.range(25., 620.), rng.range(120., 440.)];
        }
        if self.blink <= 0. {
            self.blink = rng.range(0.025, 0.18);
            self.alpha = if rng.below(2) == 0 { 0. } else { 1. };
        }
    }
    pub fn render_ui(&self, r: &mut Renderer) {
        let ram = std::str::from_utf8(&self.ram[..self.ram_len]).unwrap_or("RAM OVERFLOW");
        r.label_alpha(20., 165., 1.4, ram, [1., 0., 0.], 1.);
        if self.remaining > 0. {
            r.label_alpha(
                self.xy[0],
                self.xy[1],
                2.2,
                LOGS[self.log],
                [1., 0., 0.],
                self.alpha,
            );
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn actor(
        &self,
        r: &mut Renderer,
        p: Vec3,
        yaw: f32,
        scale: f32,
        color: [f32; 3],
        corrupt: bool,
        phase: f32,
        anchor: Option<usize>,
        mesh_override: Option<GpuMesh>,
        texture_override: Option<Texture>,
    ) {
        if let Some(mesh) = mesh_override.or(self.actor) {
            r.chaos_model(
                mesh,
                [p.x, p.y, p.z],
                yaw,
                scale,
                if mesh_override.is_some() {
                    [1.; 3]
                } else {
                    color
                },
                if mesh_override.is_some() {
                    None
                } else {
                    texture_override.or(if corrupt { self.missing } else { None })
                },
                if mesh_override.is_some() { 0. } else { phase },
                anchor,
            );
        }
    }
}

pub struct ChaosManager {
    pub spawner: SchizoSpawner,
    pub physics: PhysicsSaboteur,
    pub visuals: RenderSaboteur,
    pub absurd: absurd::AbsurdDirector,
    random: Rng,
    render_random: Rng,
}
impl ChaosManager {
    pub fn new(seed: u64) -> Self {
        let mut random = Rng::new(seed);
        let physics = PhysicsSaboteur::new(&mut random);
        let visuals = RenderSaboteur::new(&mut random);
        Self {
            spawner: SchizoSpawner::new(),
            physics,
            visuals,
            absurd: absurd::AbsurdDirector::new(),
            random,
            render_random: Rng::new(seed ^ 0x9e3779b97f4a7c15),
        }
    }
    pub fn initialize(&mut self, r: &mut Renderer) {
        self.visuals.initialize(r);
        self.absurd.initialize(r);
        println!("ABSURD scenes={}", absurd::AbsurdDirector::scene_count());
    }
    pub fn next_absurd(&mut self, world: &World, p: Vec3, forward: Vec3) {
        self.absurd.advance(world, p, forward, &mut self.random);
    }
    pub fn after_glitch(&mut self, world: &World, p: Vec3, forward: Vec3) {
        self.absurd.after_glitch(world, p, forward, &mut self.random);
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        world: &World,
        cars: &mut [Car],
        npcs: &mut [Npc],
        player: Vec3,
        eye: Vec3,
        forward: Vec3,
        dt: f32,
    ) {
        if !dt.is_finite() || dt <= 0. {
            return;
        }
        let dt = dt.min(0.1);
        self.visuals.update(&mut self.random, dt);
        self.absurd.update(
            world,
            cars,
            npcs,
            player,
            eye,
            forward,
            &mut self.physics,
            &mut self.random,
            dt,
        );
        self.absurd.compensate_gravity(&mut self.physics, dt);
        if self.absurd.gravity_off() {
            for i in 0..CAPACITY {
                if self.spawner.kind[i] != Entity::Empty && !self.spawner.frozen[i] {
                    self.spawner.velocity[i].y += 18. * dt;
                }
            }
        }
        self.spawner
            .update(world, player, eye, forward, &mut self.random, dt);
        self.physics
            .update(&self.spawner, world, cars, npcs, &mut self.random, dt);
    }
    pub fn on_shot(&mut self, car_count: usize, npc_count: usize) {
        let cars = car_count.min(BRIDGE_CAPACITY);
        let npcs = npc_count.min(BRIDGE_CAPACITY);
        let special = self
            .spawner
            .kind
            .iter()
            .filter(|&&k| k != Entity::Empty && k != Entity::GachiTitan)
            .count();
        let total = cars + npcs + special;
        if total == 0 {
            return;
        }
        let selected = self.random.below(total as u32) as usize;
        let factor = if self.random.below(2) == 0 { 0.5 } else { 2. };
        if selected < cars {
            self.physics.car_scale[selected] =
                (self.physics.car_scale[selected] * factor).clamp(0.125, 8.);
        } else if selected < cars + npcs {
            let i = selected - cars;
            self.physics.npc_scale[i] = (self.physics.npc_scale[i] * factor).clamp(0.125, 8.);
        } else {
            let mut n = selected - cars - npcs;
            for i in 0..CAPACITY {
                if self.spawner.kind[i] != Entity::Empty
                    && self.spawner.kind[i] != Entity::GachiTitan
                {
                    if n == 0 {
                        self.spawner.scale[i] = (self.spawner.scale[i] * factor).clamp(0.125, 8.);
                        break;
                    }
                    n -= 1;
                }
            }
        }
    }
    pub fn on_c4(&mut self, center: Vec3, cars: &[Car], npcs: &[Npc]) {
        fn impulse(center: Vec3, p: Vec3) -> Option<Vec3> {
            let d = sub(center, p);
            let distance = dot(d, d).sqrt();
            if distance > 0.001 && distance < 50. {
                let n = direction(d);
                let f = 120. * (1. - distance / 50.);
                Some(Vec3::new(n.x * f, n.y * f, n.z * f))
            } else {
                None
            }
        }
        for (i, c) in cars.iter().take(BRIDGE_CAPACITY).enumerate() {
            if let Some(v) = impulse(center, c.pos) {
                self.physics.launch(i, c.pos, v, 1.);
            }
        }
        for (i, n) in npcs.iter().take(BRIDGE_CAPACITY).enumerate() {
            if let Some(v) = impulse(center, n.pos) {
                self.physics.npc_airborne[i] = true;
                self.physics.npc_position[i] = n.pos;
                self.physics.npc_velocity[i] = v;
            }
        }
        for i in 0..CAPACITY {
            if self.spawner.kind[i] != Entity::Empty && !self.spawner.frozen[i] {
                if let Some(v) = impulse(center, self.spawner.position[i]) {
                    self.spawner.velocity[i] = v;
                    self.spawner.drift[i] = Vec3::new(v.x, 0., v.z);
                }
            }
        }
    }
    fn anchor(&mut self) -> Option<usize> {
        if self.render_random.below(100) < 5 {
            Some(self.render_random.below(216) as usize)
        } else {
            None
        }
    }
    pub fn render(&mut self, r: &mut Renderer, eye: Vec3) {
        r.set_chaos_crouch(self.absurd.crouch());
        for i in 0..CAPACITY {
            let kind = self.spawner.kind[i];
            let p = self.spawner.position[i];
            if kind == Entity::Empty || p.distance(eye) > 180. {
                continue;
            }
            let color = match kind {
                Entity::PinkCopOverflow => [1., 0., 1.],
                Entity::Peasant18thCentury => [0.65, 0.48, 0.24],
                _ => [1.; 3],
            };
            let phase = if kind == Entity::PinkCopOverflow && !self.spawner.infected[i] {
                self.spawner.phase[i]
            } else {
                0.
            };
            let anchor = self.anchor();
            self.visuals.actor(
                r,
                p,
                self.spawner.yaw[i],
                self.spawner.scale[i],
                color,
                kind == Entity::GachiTitan || kind == Entity::PiskaZaGuboy,
                phase,
                anchor,
                self.absurd.npc_mesh(),
                None,
            );
        }
        self.absurd.render(r, eye);
    }
    pub fn render_npc(&mut self, r: &mut Renderer, i: usize, n: &Npc) {
        r.set_chaos_crouch(self.absurd.crouch());
        let scale = self.physics.npc_scale.get(i).copied().unwrap_or(1.);
        let corrupt = self.visuals.npc_corrupt.get(i).copied().unwrap_or(false);
        let infected = self.physics.npc_infected.get(i).copied().unwrap_or(false);
        let anchor = self.anchor();
        self.visuals.actor(
            r,
            n.pos,
            n.heading,
            scale,
            n.color(),
            corrupt,
            if infected {
                0.
            } else {
                n.phase * 9. * n.motion
            },
            anchor,
            self.absurd.npc_mesh(),
            self.absurd.potato_texture(i),
        );
    }
    pub fn car_material(&self, r: &mut Renderer, i: usize) {
        let scale = self.physics.car_scale.get(i).copied().unwrap_or(1.);
        let texture = if self.visuals.car_corrupt.get(i).copied().unwrap_or(false) {
            self.visuals.missing
        } else {
            None
        };
        r.set_noodle_cars(self.absurd.noodle_cars());
        r.chaos_car_material(scale, self.absurd.potato_texture(i).or(texture));
    }
    pub fn smoke_setup(&mut self, world: &World, p: Vec3) {
        self.spawner.initialize(world, p);
        self.absurd.force(3, world, p, Vec3::new(0., 0., 1.));
        self.spawner.peasant_wait = 0.;
        self.visuals.wait = 0.;
        self.physics.frames = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npcs::Role;
    const DT: f32 = 1. / 60.;

    #[test]
    #[ignore = "requires a current SDL display; run with Xvfb and SDL_AUDIODRIVER=dummy"]
    fn warmed_render_and_ui_do_not_allocate() -> Result<(), String> {
        let mut renderer = Renderer::new()?;
        let world = World::new();
        let p = Vec3::new(80., 1., 0.);
        let mut chaos = ChaosManager::new(23);
        chaos.initialize(&mut renderer);
        chaos.smoke_setup(&world, p);
        chaos.visuals.update(&mut chaos.random, DT);
        for _ in 0..2 {
            renderer.begin([80., 4., 12.], 0., 0.2, 1.);
            renderer.glitch(0., 0., 0.);
            chaos.render(&mut renderer, p);
            renderer.ui();
            chaos.visuals.render_ui(&mut renderer);
            chaos.absurd.render_ui(&mut renderer, p, 0., 0.2);
            renderer.end();
        }
        renderer.begin([80., 4., 12.], 0., 0.2, 1.);
        let allocations = crate::allocation_probe::count(|| {
            chaos.render(&mut renderer, p);
            renderer.ui();
            chaos.visuals.render_ui(&mut renderer);
            chaos.absurd.render_ui(&mut renderer, p, 0., 0.2);
        });
        renderer.end();
        renderer.validate()?;
        assert_eq!(allocations, 0);
        for source in [6, 31, 36, 45, 94, 100] {
            chaos.absurd.force(source, &world, p, Vec3::new(0., 0., 1.));
            renderer.begin([80., 4., 12.], 0., 0.2, 1.);
            let allocations = crate::allocation_probe::count(|| {
                chaos.render(&mut renderer, p);
                renderer.ui();
                chaos.absurd.render_ui(&mut renderer, p, 0., 0.2);
            });
            renderer.end();
            renderer.validate()?;
            assert_eq!(allocations, 0);
        }
        Ok(())
    }

    #[test]
    fn missing_texture_has_exact_checker_pixels() {
        let bytes = RenderSaboteur::missing_pixels();
        assert_eq!(bytes.len(), 16 * 16 * 3);
        for y in 0..16 {
            for x in 0..16 {
                let offset = (y * 16 + x) * 3;
                assert_eq!(
                    &bytes[offset..offset + 3],
                    if (x / 4 + y / 4) % 2 == 0 {
                        &[255, 0, 255]
                    } else {
                        &[0, 0, 0]
                    }
                );
            }
        }
    }

    #[test]
    fn stalkers_freeze_in_front_and_spin_only_behind() {
        let world = World::new();
        let mut rng = Rng::new(7);
        let mut pool = SchizoSpawner::new();
        let player = Vec3::new(0., 1., 0.);
        pool.initialize(&world, player);
        pool.activate(STALKERS.start, Entity::PiskaZaGuboy, Vec3::new(0., 1., 10.));
        pool.update(&world, player, player, Vec3::new(0., 0., 1.), &mut rng, DT);
        assert_eq!(pool.position[STALKERS.start].z, 10.);
        assert_eq!(pool.yaw[STALKERS.start], 0.);
        pool.update(&world, player, player, Vec3::new(0., 0., -1.), &mut rng, DT);
        assert!(pool.position[STALKERS.start].z < 10.);
        assert!(pool.yaw[STALKERS.start] > 0.);
    }

    #[test]
    fn landing_peasant_freezes_and_infects_cop() {
        let world = World::new();
        let mut rng = Rng::new(8);
        let mut pool = SchizoSpawner::new();
        let p = Vec3::new(80., world.surface(80., 0.) + 1., 0.);
        pool.initialize(&world, p);
        pool.activate(PEASANTS.start, Entity::Peasant18thCentury, p);
        pool.activate(COPS.start, Entity::PinkCopOverflow, p);
        pool.update(&world, p, p, Vec3::new(0., 0., 1.), &mut rng, DT);
        assert!(pool.frozen[PEASANTS.start]);
        assert!(pool.infected[COPS.start]);
        let before = pool.position[PEASANTS.start];
        pool.update(&world, p, p, Vec3::new(0., 0., 1.), &mut rng, DT);
        assert_eq!(pool.position[PEASANTS.start].y, before.y);
        assert!(pool.blocks(p, 0.45));
    }

    #[test]
    fn titan_contact_launches_real_vehicle_and_c4_pulls_inward() {
        let world = World::new();
        let mut chaos = ChaosManager::new(9);
        let p = Vec3::new(80., 1., 0.);
        chaos.spawner.initialize(&world, p);
        let mut cars = [Car::new(2, chaos.spawner.position[0])];
        chaos.physics.update(
            &chaos.spawner,
            &world,
            &mut cars,
            &mut [],
            &mut chaos.random,
            DT,
        );
        assert!(cars[0].vertical_speed > 900.);
        assert!(cars[0].pos.y > chaos.spawner.position[0].y);
        assert_eq!(chaos.spawner.mass[0], f32::MAX);
        cars[0].pos = Vec3::new(90., 1., 0.);
        chaos.on_c4(Vec3::new(80., 1., 0.), &cars, &[]);
        assert!(chaos.physics.car_velocity[0].x < 0.);
    }

    #[test]
    fn launch_interval_is_between_300_and_500_frames() {
        let world = World::new();
        let mut chaos = ChaosManager::new(13);
        let mut cars = [Car::new(2, Vec3::new(80., 0.65, 0.))];
        let interval = chaos.physics.frames;
        assert!((300..=500).contains(&interval));
        for _ in 1..interval {
            chaos.physics.update(
                &chaos.spawner,
                &world,
                &mut cars,
                &mut [],
                &mut chaos.random,
                DT,
            );
        }
        assert!(!chaos.physics.car_airborne[0]);
        chaos.physics.update(
            &chaos.spawner,
            &world,
            &mut cars,
            &mut [],
            &mut chaos.random,
            DT,
        );
        assert!(chaos.physics.car_airborne[0]);
        assert!(cars[0].vertical_speed.abs() > 9900.);
        assert!((300..=500).contains(&chaos.physics.frames));
    }

    #[test]
    fn observed_wall_phases_and_unobserved_wall_blocks() {
        let collider = buildings::Collider {
            p: Vec3::new(0., 1., 10.),
            size: Vec3::new(10., 3., 1.),
            movable: false,
            breakable: false,
        };
        assert!(!PhysicsSaboteur::wall_active(
            Vec3::new(0., 1., 0.),
            Vec3::new(0., 0., 1.),
            &collider
        ));
        assert!(PhysicsSaboteur::wall_active(
            Vec3::new(0., 1., 0.),
            Vec3::new(0., 0., -1.),
            &collider
        ));
    }

    #[test]
    fn pools_saturate_and_update_events_do_not_allocate() {
        let world = World::new();
        let mut chaos = ChaosManager::new(42);
        let mut cars = [Car::new(2, Vec3::new(80., 0.65, 0.))];
        let mut npcs = [Npc::new(Role::Police, 82., 0.)];
        let p = Vec3::new(80., 1., 0.);
        let allocations = crate::allocation_probe::count(|| {
            for i in 0..7500 {
                chaos.update(
                    &world,
                    &mut cars,
                    &mut npcs,
                    p,
                    p,
                    Vec3::new(0., 0., -1.),
                    0.1,
                );
                if i % 7 == 0 {
                    chaos.on_shot(cars.len(), npcs.len());
                }
                if i % 31 == 0 {
                    chaos.on_c4(p, &cars, &npcs);
                }
            }
        });
        assert_eq!(allocations, 0);
        assert_eq!(
            chaos
                .spawner
                .kind
                .iter()
                .filter(|&&k| k == Entity::GachiTitan)
                .count(),
            15
        );
        assert_eq!(
            chaos
                .spawner
                .kind
                .iter()
                .filter(|&&k| k == Entity::PinkCopOverflow)
                .count(),
            64
        );
        assert_eq!(
            chaos
                .spawner
                .kind
                .iter()
                .filter(|&&k| k == Entity::Peasant18thCentury)
                .count(),
            64
        );
        for i in TITANS {
            assert_eq!(chaos.spawner.scale[i], 2.5);
        }
        assert!(chaos
            .physics
            .car_scale
            .iter()
            .all(|s| (0.125..=8.).contains(s)));
        assert!(cars[0].pos.y.is_finite());
    }

    #[test]
    fn fake_ram_saturates_rolls_back_and_accumulates_again() {
        let mut rng = Rng::new(19);
        let mut ui = RenderSaboteur::new(&mut rng);
        for _ in 0..61 {
            ui.update(&mut rng, 1. / 60.);
        }
        assert_eq!(ui.fake_gb, 1);
        ui.fake_gb = 999_999_000;
        ui.write_ram();
        assert_eq!(&ui.ram[..ui.ram_len], b"999999 TB RAM USED");
        for _ in 0..61 {
            ui.update(&mut rng, 1. / 60.);
        }
        assert_eq!(ui.fake_gb, 999_999_000);
        ui.ram_reset_in = 0.05;
        ui.update(&mut rng, 0.1);
        assert_eq!(ui.fake_gb, 0);
        assert_eq!(&ui.ram[..ui.ram_len], b"0 GB RAM USED");
        assert!((20. ..=40.).contains(&ui.ram_reset_in));
        for _ in 0..61 {
            ui.update(&mut rng, 1. / 60.);
        }
        assert_eq!(ui.fake_gb, 1);
        let mut rollbacks = 0;
        for _ in 0..1200 {
            let before = ui.fake_gb;
            ui.update(&mut rng, 0.1);
            if ui.fake_gb < before {
                rollbacks += 1;
                assert_eq!(ui.fake_gb, 0);
            }
            assert!(ui.log < LOGS.len());
            assert!(ui.alpha == 0. || ui.alpha == 1.);
        }
        assert!(rollbacks >= 2);
    }
}
