//! Cosmetic corruption bursts. This RNG never changes gameplay randomness or saves.
pub(crate) struct Glitches {
    random: u64,
    wait: f32,
    remaining: f32,
    elapsed: f32,
    seed: f32,
}
impl Glitches {
    pub fn new(seed: u64) -> Self {
        let mut value = Self {
            random: seed.max(1),
            wait: 0.,
            remaining: 0.,
            elapsed: 0.,
            seed: 0.,
        };
        value.wait = 12. + value.next() * 18.;
        value
    }
    fn next(&mut self) -> f32 {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 7;
        self.random ^= self.random << 17;
        (self.random >> 40) as f32 / 16_777_216.
    }
    pub fn burst(&mut self) {
        self.remaining = 4. + self.next() * 5.;
        self.elapsed = 0.;
        self.seed = self.next() * 63.;
    }
    pub fn tick(&mut self, dt: f32) {
        if self.remaining > 0. {
            self.remaining = (self.remaining - dt).max(0.);
            self.elapsed += dt;
            if self.remaining == 0. {
                self.wait = 18. + self.next() * 37.;
            }
        } else {
            self.wait -= dt;
            if self.wait <= 0. {
                self.burst();
            }
        }
    }
    pub fn parameters(&self) -> [f32; 3] {
        [
            (self.elapsed * 8.).min(1.) * (self.remaining * 5.).min(1.),
            self.elapsed,
            self.seed,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bursts_start_end_and_recur_with_varied_timing() {
        let mut g = Glitches::new(42);
        let mut starts = Vec::new();
        let mut was_active = false;
        for tick in 0..36_000 {
            g.tick(1. / 60.);
            let [amount, _, seed] = g.parameters();
            assert!((0. ..=1.).contains(&amount) && seed.is_finite());
            let active = amount > 0.;
            if active && !was_active {
                starts.push(tick);
            }
            was_active = active;
        }
        assert!(starts.len() > 5);
        assert!(starts[0] >= 12 * 60 && starts[0] <= 30 * 60 + 1);
        assert!(starts.windows(3).any(|w| w[1] - w[0] != w[2] - w[1]));
    }
}
