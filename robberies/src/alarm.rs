use engine_core::Vec3;
#[derive(Clone, Copy)]
pub struct Site {
    pub name: &'static str,
    pub center: Vec3,
    pub radius: f32,
    pub reward: u32,
    pub collect: f32,
}
pub const SITES: [Site; 7] = [
    Site {
        name: "BANK",
        center: Vec3 {
            x: 320.,
            y: 1.,
            z: 35.,
        },
        radius: 55.,
        reward: 420,
        collect: 6.,
    },
    Site {
        name: "DONUT SHOP",
        center: Vec3 {
            x: 110.,
            y: 1.,
            z: 240.,
        },
        radius: 40.,
        reward: 80,
        collect: 3.,
    },
    Site {
        name: "GROCERY",
        center: Vec3 {
            x: 210.,
            y: 1.,
            z: 240.,
        },
        radius: 40.,
        reward: 120,
        collect: 4.,
    },
    Site {
        name: "FACTORY A",
        center: Vec3 {
            x: 640.,
            y: 19.,
            z: 220.,
        },
        radius: 60.,
        reward: 260,
        collect: 8.,
    },
    Site {
        name: "FACTORY B",
        center: Vec3 {
            x: 720.,
            y: 19.,
            z: 220.,
        },
        radius: 60.,
        reward: 240,
        collect: 8.,
    },
    Site {
        name: "APARTMENT A",
        center: Vec3 {
            x: -220.,
            y: 1.,
            z: 210.,
        },
        radius: 35.,
        reward: 90,
        collect: 5.,
    },
    Site {
        name: "APARTMENT B",
        center: Vec3 {
            x: -300.,
            y: 1.,
            z: 260.,
        },
        radius: 35.,
        reward: 110,
        collect: 5.,
    },
];
#[derive(Default)]
pub struct Alarm {
    pub site: Option<usize>,
    pub remaining: f32,
    pub progress: f32,
    pub loot: u32,
    pub dispatched: bool,
    pub cooldown: [f32; 7],
    pub hits: [u8; 7],
    pub collecting: bool,
    pub capacity: u32,
}
#[derive(Debug, PartialEq)]
pub enum Outcome {
    None,
    Escaped(u32),
    Dispatch(usize),
}
impl Alarm {
    pub fn start(&mut self, id: usize) -> bool {
        if id >= SITES.len() || self.site.is_some() || self.cooldown[id] > 0. {
            return false;
        }
        self.site = Some(id);
        self.remaining = 30.;
        self.progress = 0.;
        self.loot = 0;
        self.dispatched = false;
        true
    }
    pub fn case(id: usize) -> Vec3 {
        let p = SITES[id].center;
        Vec3::new(
            p.x,
            if id == 3 || id == 4 {
                terrain::height(p.x, p.z).max(0.) + 1.
            } else {
                p.y
            },
            p.z - 3.,
        )
    }
    pub fn strike(&mut self, id: usize) -> bool {
        if id >= SITES.len() || self.cooldown[id] > 0. || self.site.is_some_and(|site| site != id) {
            return false;
        }
        if self.site.is_none() {
            self.start(id);
        }
        self.hits[id] = (self.hits[id] + 1).min(3);
        true
    }
    pub fn tick(&mut self, p: Vec3, dt: f32) -> Outcome {
        for c in &mut self.cooldown {
            *c = (*c - dt).max(0.);
        }
        let Some(id) = self.site else {
            return Outcome::None;
        };
        let site = SITES[id];
        self.remaining = (self.remaining - dt).max(0.);
        if self.collecting
            && self.hits[id] >= 3
            && p.distance(Self::case(id)) < 2.6
            && (p.y - Self::case(id).y).abs() < 2.
        {
            self.progress += dt;
            if self.progress >= site.collect {
                let weight = if id == 0 || id == 3 || id == 4 {
                    12
                } else if id >= 5 {
                    6
                } else {
                    4
                };
                self.loot = site.reward * self.capacity.clamp(8, 18).min(weight) / weight;
            }
        }
        // The deadline is evaluated before escape: crossing after zero keeps the pursuit active.
        if self.remaining <= 0. && !self.dispatched {
            self.dispatched = true;
            return Outcome::Dispatch(id);
        }
        if p.distance(site.center) > site.radius && self.loot > 0 && !self.dispatched {
            let loot = self.loot;
            self.finish();
            return Outcome::Escaped(loot);
        }
        Outcome::None
    }
    pub fn finish(&mut self) {
        if let Some(id) = self.site {
            self.cooldown[id] = 300. + (id as f32 * 97.).rem_euclid(601.);
            self.hits[id] = 0;
        }
        self.site = None;
        self.loot = 0;
        self.progress = 0.;
        self.dispatched = false;
    }
    pub fn cancel(&mut self) {
        self.finish();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn three_sites_pay_only_after_escape_and_once() {
        for (id, _) in SITES.iter().enumerate() {
            let mut a = Alarm::default();
            assert!(a.start(id));
            a.hits[id] = 3;
            a.collecting = true;
            a.capacity = 12;
            assert_eq!(
                a.tick(Alarm::case(id), SITES[id].collect + 0.1),
                Outcome::None
            );
            assert_eq!(a.loot, SITES[id].reward);
            let p = Vec3::new(
                SITES[id].center.x + SITES[id].radius + 1.,
                1.,
                SITES[id].center.z,
            );
            assert_eq!(a.tick(p, 1.), Outcome::Escaped(SITES[id].reward));
            assert_eq!(a.tick(p, 1.), Outcome::None);
            assert!(!a.start(id));
        }
    }
    #[test]
    fn deadline_dispatches_and_late_escape_never_pays() {
        let mut a = Alarm::default();
        a.start(0);
        assert_eq!(a.tick(SITES[0].center, 30.), Outcome::Dispatch(0));
        assert_eq!(a.tick(Vec3::new(500., 1., 35.), 1.), Outcome::None);
        assert!(a.dispatched);
        a.cancel();
        assert_eq!(a.loot, 0);
    }
    #[test]
    fn cannot_collect_from_outside_and_deadline_has_priority() {
        let mut a = Alarm::default();
        a.start(1);
        assert_eq!(a.tick(Vec3::new(500., 1., 35.), 29.), Outcome::None);
        assert_eq!(a.loot, 0);
        assert_eq!(a.tick(SITES[1].center, 1.), Outcome::Dispatch(1));
    }
}
