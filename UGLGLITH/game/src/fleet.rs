//! Active-play timer for replacing the complete vehicle fleet.

const REFRESH_SECONDS: f64 = 450.0;

pub struct FleetCycle {
    elapsed: f64,
    seed: u64,
    generation: u64,
}

impl FleetCycle {
    pub fn new(seed: u64) -> Self {
        Self {
            elapsed: 0.0,
            seed,
            generation: 0,
        }
    }

    /// Returns a new spawn seed once per 450 seconds of active simulation time.
    pub fn tick(&mut self, dt: f32) -> Option<u64> {
        if !dt.is_finite() || dt <= 0.0 {
            return None;
        }
        self.elapsed += f64::from(dt);
        if self.elapsed < REFRESH_SECONDS {
            return None;
        }
        self.elapsed -= REFRESH_SECONDS;
        self.generation = self.generation.wrapping_add(1);
        Some(self.seed.wrapping_add(self.generation))
    }

    #[cfg(test)]
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_is_measured_in_active_time_and_changes_seed_each_round() {
        let mut cycle = FleetCycle::new(42);
        assert_eq!(cycle.tick(449.0), None);
        assert_eq!(cycle.tick(0.0), None);
        assert_eq!(cycle.tick(f32::NAN), None);
        let first = cycle.tick(1.0).unwrap();
        assert!((cycle.elapsed() - 0.0).abs() < 0.001);
        let second = cycle.tick(450.0).unwrap();
        assert_ne!(first, second);
        assert!((cycle.elapsed() - 0.0).abs() < 0.001);
    }
}
