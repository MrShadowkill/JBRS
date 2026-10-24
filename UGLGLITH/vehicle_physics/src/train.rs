//! Longitudinal train dynamics in SI units, integrated independently of the rail geometry.

const GRAVITY: f32 = 9.81;
const ROLLING_COEFFICIENT: f32 = 0.0018;

#[derive(Clone, Debug)]
pub struct Train {
    pub distance: f32,
    pub speed: f32,
    pub mass_kg: f32,
    pub traction_limit_n: f32,
    pub power_w: f32,
    pub brake_limit_n: f32,
    pub max_speed: f32,
}

impl Default for Train {
    fn default() -> Self {
        Self {
            distance: 0.0,
            speed: 11.2,
            mass_kg: 1_500_000.0,
            traction_limit_n: 750_000.0,
            power_w: 10_000_000.0,
            brake_limit_n: 650_000.0,
            max_speed: 27.8,
        }
    }
}

impl Train {
    pub fn cruise_step(&mut self, dt: f32, grade: f32, target_speed: f32) {
        let target = target_speed.clamp(0.0, self.max_speed);
        let error = target - self.speed;
        let throttle = (error * 0.45).clamp(0.0, 1.0);
        let brake = ((-error - 0.15) * 1.25).clamp(0.0, 1.0);
        self.step_forces(dt, grade, throttle, brake);
    }

    pub fn step_forces(&mut self, dt: f32, grade: f32, throttle: f32, brake: f32) {
        if !dt.is_finite() || dt <= 0.0 || !self.mass_kg.is_finite() || self.mass_kg <= 0.0 {
            return;
        }
        let dt = dt.min(0.1);
        let grade = if grade.is_finite() {
            grade.clamp(-0.12, 0.12)
        } else {
            0.0
        };
        let old_speed = self.speed.max(0.0);
        let traction = (self.traction_limit_n.max(0.0) * throttle.clamp(0.0, 1.0))
            .min(self.power_w.max(0.0) / old_speed.max(1.0));
        let resist = self.mass_kg * GRAVITY * ROLLING_COEFFICIENT + 180.0 * old_speed * old_speed;
        let grade_force = self.mass_kg * GRAVITY * grade;
        let brake_force = self.brake_limit_n.max(0.0) * brake.clamp(0.0, 1.0);
        let acceleration = (traction - resist - grade_force - brake_force) / self.mass_kg;
        self.speed = (old_speed + acceleration * dt).clamp(0.0, self.max_speed.max(0.0));
        self.distance += (old_speed + self.speed) * 0.5 * dt;
    }

    pub fn emergency_brake(&mut self) {
        self.speed = (self.speed - self.brake_limit_n / self.mass_kg.max(1.0) * 0.1).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mass_traction_and_inertia_shape_acceleration() {
        let mut loaded = Train::default();
        let mut light = Train {
            mass_kg: 750_000.0,
            ..Train::default()
        };
        loaded.speed = 0.0;
        light.speed = 0.0;
        loaded.step_forces(1.0, 0.0, 1.0, 0.0);
        light.step_forces(1.0, 0.0, 1.0, 0.0);
        assert!(loaded.speed > 0.0);
        assert!(light.speed > loaded.speed);

        let coasting_speed = loaded.speed;
        loaded.step_forces(1.0, 0.0, 0.0, 0.0);
        assert!(loaded.speed < coasting_speed);
        assert!(loaded.speed > coasting_speed - 0.1);
        assert!(loaded.distance > 0.0);
    }

    #[test]
    fn uphill_slows_down_downhill_accelerates_and_braking_stops_without_reversing() {
        let mut uphill = Train::default();
        let mut downhill = Train::default();
        uphill.step_forces(1.0, 0.03, 0.2, 0.0);
        downhill.step_forces(1.0, -0.03, 0.2, 0.0);
        assert!(uphill.speed < downhill.speed);

        let mut braking = Train::default();
        for _ in 0..3600 {
            braking.step_forces(1.0 / 60.0, 0.0, 0.0, 1.0);
        }
        assert_eq!(braking.speed, 0.0);
        assert!(braking.distance > 0.0);
    }

    #[test]
    fn cruise_control_stays_bounded_on_grade_changes() {
        let mut train = Train::default();
        for i in 0..1800 {
            let grade = if i < 600 {
                0.02
            } else if i < 1200 {
                -0.02
            } else {
                0.0
            };
            train.cruise_step(1.0 / 60.0, grade, 11.2);
        }
        assert!(train.speed > 0.0 && train.speed < 14.0);
        assert!(train.distance > 100.0);
    }
}
