use engine_core::Vec3;
use vehicles::{Car, CATALOG};
pub fn step(car: &mut Car, throttle: f32, steer: f32, brake: bool, dt: f32) -> Vec3 {
    let dt = dt.clamp(0., 0.1);
    let s = &CATALOG[car.kind];
    car.speed += throttle * s.acceleration * (1. + car.engine_level as f32 * 0.18) * dt;
    car.speed *= (-dt
        * if brake {
            7. + car.brake_level as f32 * 1.5
        } else {
            0.22
        })
    .exp();
    if brake && car.speed.abs() < 0.15 {
        car.speed = 0.;
    }
    let maximum = s.max_speed * (1. + car.engine_level as f32 * 0.08);
    car.speed = car.speed.clamp(-maximum * 0.35, maximum);
    car.angle +=
        steer * car.speed * dt * 0.045 * s.grip * (1. + car.suspension_level as f32 * 0.12)
            / (1. + car.speed.abs() * 0.04);
    Vec3::new(
        car.pos.x + car.angle.sin() * car.speed * dt,
        car.pos.y,
        car.pos.z + car.angle.cos() * car.speed * dt,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn engine_upgrade_accelerates_and_brakes_upgrade_stops_faster() {
        let mut stock = Car::new(0, Vec3::default());
        let mut tuned = Car::new(0, Vec3::default());
        tuned.engine_level = 2;
        for _ in 0..60 {
            step(&mut stock, 1., 0., false, 1. / 60.);
            step(&mut tuned, 1., 0., false, 1. / 60.);
        }
        assert!(tuned.speed > stock.speed);
        stock.speed = 30.;
        tuned.speed = 30.;
        tuned.brake_level = 2;
        step(&mut stock, 0., 0., true, 0.1);
        step(&mut tuned, 0., 0., true, 0.1);
        assert!(tuned.speed < stock.speed);
    }
    #[test]
    fn braking_large_dt_does_not_reverse_or_lose_elevation() {
        let mut c = Car::new(0, Vec3::new(0., 5., 0.));
        c.speed = 30.;
        let p = step(&mut c, 0., 0., true, 1.);
        assert!(c.speed >= 0. && c.speed < 30.);
        assert_eq!(p.y, 5.);
    }
    #[test]
    fn bounded_speed() {
        let mut c = Car::new(0, Vec3::default());
        for _ in 0..6000 {
            c.pos = step(&mut c, 1., 0., false, 1. / 60.);
        }
        assert!(c.speed <= 43.);
        assert!(c.speed > 30.);
    }
}
