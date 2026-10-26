use engine_core::Vec3;
#[derive(Default)]
pub struct Bank {
    pub progress: f32,
    pub loot: u32,
    pub cooldown: f32,
    pub active: bool,
    pub timer: f32,
}
impl Bank {
    pub fn start(&mut self) -> bool {
        if self.cooldown <= 0. && !self.active && self.loot == 0 {
            self.active = true;
            self.timer = 70.;
            self.progress = 0.;
            true
        } else {
            false
        }
    }
    pub fn tick(&mut self, p: Vec3, dt: f32) -> bool {
        self.cooldown = (self.cooldown - dt).max(0.);
        if self.active {
            self.timer -= dt;
            if self.timer <= 0. {
                self.cancel();
                return false;
            }
            if p.distance(Vec3::new(320., 0., 44.)) < 7. {
                self.progress += dt;
                if self.progress >= 8. {
                    self.loot = 2500;
                    self.active = false;
                    return true;
                }
            }
        }
        false
    }
    pub fn deliver(&mut self) -> u32 {
        let loot = self.loot;
        self.loot = 0;
        if loot > 0 {
            self.cooldown = 90.;
        }
        loot
    }
    pub fn cancel(&mut self) {
        self.active = false;
        self.loot = 0;
        self.progress = 0.;
        self.cooldown = 30.;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delivery_once() {
        let mut b = Bank::default();
        assert!(b.start());
        assert!(b.tick(Vec3::new(320., 0., 44.), 8.));
        assert_eq!(b.deliver(), 2500);
        assert_eq!(b.deliver(), 0);
    }
}
pub mod alarm;
