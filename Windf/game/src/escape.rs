#[derive(Default)]
pub struct Cuffs {
    pub active: bool,
    pub remaining: f32,
    pub required: u32,
    pub success: u32,
    pub marker: f32,
    pub target: f32,
    direction: f32,
    random: u64,
}
impl Cuffs {
    pub fn start(&mut self, stars: u32, seed: &mut u64) {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        *self = Self {
            active: true,
            remaining: 5.,
            required: stars.clamp(1, 5),
            success: 0,
            marker: 0.,
            target: 0.,
            direction: 1.,
            random: *seed,
        };
        self.next();
    }
    pub fn width(&self) -> f32 {
        0.24 - (self.success as f32) * 0.018
    }
    fn next(&mut self) {
        self.random = self
            .random
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        self.target = 0.05 + (self.random >> 32) as f32 / u32::MAX as f32 * (0.9 - self.width());
    }
    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.active {
            return false;
        }
        self.remaining = (self.remaining - dt).max(0.);
        self.marker += self.direction * dt * (0.65 + self.success as f32 * 0.16);
        if self.marker >= 1. {
            self.marker = 2. - self.marker;
            self.direction = -1.;
        }
        if self.marker <= 0. {
            self.marker = -self.marker;
            self.direction = 1.;
        }
        self.remaining <= 0.
    }
    pub fn hit(&mut self) -> bool {
        if !self.active || self.remaining <= 0. {
            return false;
        }
        if (self.target..=self.target + self.width()).contains(&self.marker) {
            self.success += 1;
            if self.success >= self.required {
                self.active = false;
                return true;
            }
            self.next();
        }
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_star_requires_exact_hits() {
        for n in 1..=5 {
            let mut c = Cuffs::default();
            c.start(n, &mut 42);
            for hit in 1..=n {
                c.marker = c.target + c.width() / 2.;
                assert_eq!(c.hit(), hit == n);
            }
            assert!(!c.active);
        }
    }
    #[test]
    fn misses_and_timeout() {
        let mut c = Cuffs::default();
        c.start(3, &mut 5);
        c.marker = 0.;
        assert!(!c.hit());
        assert_eq!(c.success, 0);
        assert!(c.tick(5.));
        assert!(!c.hit());
    }
}
