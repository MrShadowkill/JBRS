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
    car.angle += steer * car.speed * dt * 0.22 * s.grip * (1. + car.suspension_level as f32 * 0.12)
        / (1. + car.speed.abs() * 0.04);
    Vec3::new(
        car.pos.x + car.angle.sin() * car.speed * dt,
        car.pos.y,
        car.pos.z + car.angle.cos() * car.speed * dt,
    )
}
/// Swept capsule movement shared by the player and autonomous vehicles.
/// Water is deliberately absent: the ground callback supplies the riverbed.
pub fn drive(
    car: &mut Car,
    input: (f32, f32, bool),
    dt: f32,
    ground: impl Fn(f32, f32) -> f32,
    blocked: impl Fn(Vec3, f32) -> bool,
) -> bool {
    let dt = dt.clamp(0., 0.1);
    let old_angle = car.angle;
    let target = step(car, input.0, input.1, input.2, dt);
    let clear = |p: Vec3, angle: f32| {
        [-1.5, 0., 1.5].into_iter().all(|offset| {
            !blocked(
                Vec3::new(p.x + angle.sin() * offset, p.y, p.z + angle.cos() * offset),
                0.95,
            )
        })
    };
    // A steering wheel cannot rotate the chassis through a wall.
    if !clear(car.pos, car.angle) {
        car.angle = old_angle;
    }
    let count = (car.pos.distance(target) / 0.18).ceil().max(1.) as usize;
    let dx = (target.x - car.pos.x) / count as f32;
    let dz = (target.z - car.pos.z) / count as f32;
    let mut hit = false;
    for _ in 0..count {
        let mut next = Vec3::new(car.pos.x + dx, car.pos.y, car.pos.z + dz);
        let floor = ground(next.x, next.z) + 0.65;
        // Allow slopes, bridge approaches and falling into water; reject cliffs upward.
        next.y = next.y.max(floor.min(next.y + 0.32));
        if floor <= car.pos.y + 0.32 && clear(next, car.angle) {
            car.pos = next;
        } else {
            hit = true;
            for (x, z) in [(dx, 0.), (0., dz)] {
                let mut q = Vec3::new(car.pos.x + x, car.pos.y, car.pos.z + z);
                let h = ground(q.x, q.z) + 0.65;
                q.y = q.y.max(h.min(q.y + 0.32));
                if h <= car.pos.y + 0.32 && clear(q, car.angle) {
                    car.pos = q;
                }
            }
            break;
        }
    }
    if hit {
        if car.speed.abs() > 3. {
            car.damage = (car.damage + car.speed.abs() * 0.08).min(100.);
        }
        // Very weak restitution. Never add energy or bounce several metres.
        car.speed = -car.speed * 0.035;
        car.speed = car.speed.clamp(-0.7, 0.7);
    }
    let (sx, sz) = (car.angle.sin(), car.angle.cos());
    let pitch = -((ground(car.pos.x + sx * 1.5, car.pos.z + sz * 1.5)
        - ground(car.pos.x - sx * 1.5, car.pos.z - sz * 1.5))
        / 3.)
        .atan();
    let roll = ((ground(car.pos.x + sz, car.pos.z - sx) - ground(car.pos.x - sz, car.pos.z + sx))
        / 2.)
        .atan();
    for (v, goal) in car.tilt.iter_mut().zip([pitch, roll]) {
        *v += (goal.clamp(-0.35, 0.35) - *v) * (dt * 9.).min(1.);
    }
    let floor = ground(car.pos.x, car.pos.z) + 0.65;
    car.vertical_speed -= 18. * dt;
    car.pos.y += car.vertical_speed * dt;
    if car.pos.y <= floor {
        car.pos.y = floor;
        car.vertical_speed = 0.;
    }
    hit
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

#[cfg(test)]
mod contact_tests {
    use super::*;
    #[test]
    fn fast_car_cannot_tunnel_and_rebounds_weakly() {
        let mut c = Car::new(0, Vec3::new(0., 0.65, 0.));
        c.speed = 40.;
        for _ in 0..30 {
            if drive(
                &mut c,
                (0., 0., false),
                1. / 60.,
                |_, _| 0.,
                |p, r| p.z + r > 8.,
            ) {
                assert!(c.speed < 0. && c.speed.abs() <= 0.7);
                assert!(c.pos.z < 5.6);
                return;
            }
        }
        panic!("no wall contact");
    }
    #[test]
    fn riverbed_is_drivable_and_downward_motion_has_gravity() {
        let mut c = Car::new(0, Vec3::new(0., 0.65, 0.));
        c.speed = 8.;
        drive(&mut c, (0., 0., false), 1. / 60., |_, _| -4., |_, _| false);
        assert!(c.pos.y < 0.65 && c.pos.y > 0.6);
        for _ in 0..180 {
            drive(&mut c, (0., 0., false), 1. / 60., |_, _| -4., |_, _| false);
        }
        assert!((c.pos.y + 3.35).abs() < 0.001);
        assert!(c.pos.z > 5.);
    }
}
