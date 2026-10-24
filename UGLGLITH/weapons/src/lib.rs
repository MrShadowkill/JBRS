#[derive(Clone, Copy)]
pub struct Spec {
    pub name: &'static str,
    pub price: u32,
    pub magazine: u32,
    pub reserve: u32,
    pub damage: f32,
    pub range: f32,
    pub interval: f32,
    pub reload: f32,
}
pub const CATALOG: [Spec; 3] = [
    Spec {
        name: "PISTOL",
        price: 350,
        magazine: 12,
        reserve: 36,
        damage: 34.,
        range: 65.,
        interval: 0.25,
        reload: 1.4,
    },
    Spec {
        name: "SHOTGUN",
        price: 600,
        magazine: 6,
        reserve: 18,
        damage: 65.,
        range: 24.,
        interval: 0.85,
        reload: 2.1,
    },
    Spec {
        name: "RIFLE",
        price: 950,
        magazine: 24,
        reserve: 72,
        damage: 23.,
        range: 100.,
        interval: 0.12,
        reload: 1.8,
    },
];
use engine_core::Vec3;
pub struct Pistol {
    pub kind: usize,
    pub ammo: u32,
    pub reserve: u32,
    pub cooldown: f32,
    pub reload: f32,
}
impl Default for Pistol {
    fn default() -> Self {
        Self {
            kind: 0,
            ammo: 12,
            reserve: 72,
            cooldown: 0.,
            reload: 0.,
        }
    }
}
impl Pistol {
    pub fn tick(&mut self, dt: f32) {
        self.cooldown = (self.cooldown - dt).max(0.);
        if self.reload > 0. {
            self.reload -= dt;
            if self.reload <= 0. {
                let n = (CATALOG[self.kind].magazine - self.ammo).min(self.reserve);
                self.ammo += n;
                self.reserve -= n;
            }
        }
    }
    pub fn fire(&mut self) -> bool {
        if self.cooldown > 0. || self.reload > 0. || self.ammo == 0 {
            return false;
        }
        self.ammo -= 1;
        self.cooldown = CATALOG[self.kind].interval;
        true
    }
    pub fn reload(&mut self) {
        if self.reload <= 0. && self.ammo < CATALOG[self.kind].magazine && self.reserve > 0 {
            self.reload = CATALOG[self.kind].reload;
        }
    }
}
pub fn ray_distance(origin: Vec3, yaw: f32, target: Vec3) -> Option<f32> {
    ray_distance_range(origin, yaw, target, 65.)
}
pub fn ray_distance_range(origin: Vec3, yaw: f32, target: Vec3, range: f32) -> Option<f32> {
    if (target.y - origin.y).abs() > 1.8 {
        return None;
    }
    let dx = target.x - origin.x;
    let dz = target.z - origin.z;
    let along = dx * yaw.sin() - dz * yaw.cos();
    let side = dx * yaw.cos() + dz * yaw.sin();
    if along > 0. && along < range && side.abs() < 1.1 {
        Some(along)
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_weapon_uses_its_magazine_cooldown_and_reload() {
        for (id, s) in CATALOG.iter().enumerate() {
            let mut w = Pistol {
                kind: id,
                ammo: s.magazine,
                reserve: s.reserve,
                ..Default::default()
            };
            assert!(w.fire());
            assert!(!w.fire());
            w.tick(s.interval + 0.01);
            assert!(w.fire());
            w.ammo = 0;
            w.reload();
            assert!(!w.fire());
            w.tick(s.reload + 0.01);
            assert_eq!(w.ammo, s.magazine);
        }
    }
    #[test]
    fn targets_on_other_floor_are_not_hit() {
        assert!(ray_distance(Vec3::new(0., -3., 0.), 0., Vec3::new(0., 1., -5.)).is_none());
        assert!(ray_distance(Vec3::new(0., 1., 0.), 0., Vec3::new(0., 1., -5.)).is_some());
    }
    #[test]
    fn fire_and_reload() {
        let mut p = Pistol::default();
        for _ in 0..12 {
            assert!(p.fire());
            p.tick(0.3);
        }
        assert!(!p.fire());
        p.reload();
        assert!(!p.fire());
        p.tick(1.5);
        assert_eq!(p.ammo, 12);
        assert_eq!(p.reserve, 60);
    }
}
