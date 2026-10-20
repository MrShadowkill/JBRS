//! Existing road vehicles, including patrols, travel physically to incidents.
use engine_core::Vec3;
use vehicles::Car;
use world::World;
pub struct Driver {
    pub car: usize,
    pub police: bool,
    pub route: Vec<Vec3>,
    next: usize,
    pub incident: bool,
    pub pursuing: bool,
    pub parked: bool,
    pub crew: Vec<usize>,
    pub npc_ejected: bool,
    pub stuck: f32,
    pub incident_time: f32,
}
pub struct Traffic {
    pub drivers: Vec<Driver>,
    graph: roads::routing::Network,
    serial: usize,
}
impl Traffic {
    pub fn with_seed(world: &World, cars: &mut Vec<Car>, seed: u64) -> Self {
        let graph = roads::routing::Network::new(&world.roads);
        let initial_serial = (seed % graph.nodes.len().max(1) as u64) as usize;
        let spawn_nodes: Vec<_> = graph
            .nodes
            .iter()
            .copied()
            .filter(|p| !prison_parking_zone(*p))
            .collect();
        let mut drivers = Vec::new();
        for i in 0..24usize {
            if spawn_nodes.is_empty() {
                break;
            }
            let police = i % 4 == 0;
            let offset = (seed % spawn_nodes.len() as u64) as usize;
            let from = spawn_nodes[(i * 97 + 32 + offset) % spawn_nodes.len()];
            let to = spawn_nodes[(i * 173 + 240 + offset / 7) % spawn_nodes.len()];
            let Some((_, route)) = graph.route(from, to) else {
                continue;
            };
            if route.len() < 3 {
                continue;
            }
            let a = route[0];
            let b = route[1];
            let mut car = Car::new(
                if police { 2 } else { [0, 1, 4, 5][i % 4] },
                Vec3::new(a.x, world.surface(a.x, a.z) + 0.65, a.z),
            );
            car.angle = (b.x - a.x).atan2(b.z - a.z);
            if [-1.5, 0., 1.5].iter().any(|o| {
                world.blocked(
                    Vec3::new(
                        car.pos.x + car.angle.sin() * o,
                        car.pos.y,
                        car.pos.z + car.angle.cos() * o,
                    ),
                    1.,
                )
            }) || cars.iter().any(|c| c.pos.distance(car.pos) < 8.)
            {
                continue;
            }
            let id = cars.len();
            cars.push(car);
            drivers.push(Driver {
                car: id,
                police,
                route,
                next: 1,
                incident: false,
                pursuing: false,
                parked: false,
                crew: Vec::new(),
                npc_ejected: false,
                stuck: 0.,
                incident_time: 0.,
            });
        }
        Self {
            drivers,
            graph,
            serial: initial_serial,
        }
    }
    pub fn dispatch(
        &mut self,
        cars: &[Car],
        target: Vec3,
        count: usize,
        player: Option<usize>,
    ) -> Vec<(Vec<Vec3>, usize)> {
        let mut candidates = Vec::new();
        for (i, d) in self.drivers.iter().enumerate() {
            if !d.police || d.incident || Some(d.car) == player {
                continue;
            }
            if let Some((distance, route)) = self.graph.route(cars[d.car].pos, target) {
                candidates.push((distance, i, route));
            }
        }
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut selected = Vec::new();
        for (_, i, route) in candidates.into_iter().take(count) {
            let d = &mut self.drivers[i];
            d.route = route.clone();
            d.next = 0;
            d.incident = true;
            d.pursuing = true;
            d.incident_time = 0.;
            d.parked = false;
            d.stuck = 0.;
            selected.push((route, d.car));
        }
        selected
    }
    pub fn release(&mut self, car: usize) {
        if let Some(d) = self.drivers.iter_mut().find(|d| d.car == car) {
            d.parked = true;
            d.incident = false;
            d.pursuing = false;
        }
    }
    pub fn retarget_pursuit(&mut self, cars: &[Car], target: Vec3) {
        for d in &mut self.drivers {
            if !d.pursuing || d.parked {
                continue;
            }
            if let Some((_, route)) = self.graph.route(cars[d.car].pos, target) {
                d.route = route;
                d.next = 0;
                d.stuck = 0.;
            }
        }
    }
    /// Returns cars that have actually stopped at the incident, never a timed spawn.
    pub fn tick(
        &mut self,
        world: &World,
        cars: &mut [Car],
        player: Option<usize>,
        train_distance: f32,
        dt: f32,
        blocked: impl Fn(Vec3, f32) -> bool,
    ) -> Vec<usize> {
        let snapshot: Vec<_> = cars.iter().map(|c| (c.pos, c.angle)).collect();
        let mut arrived = Vec::new();
        for d in &mut self.drivers {
            if d.incident {
                d.incident_time += dt;
            }
            if Some(d.car) == player || d.parked {
                continue;
            }
            let c = &mut cars[d.car];
            while d.next < d.route.len() && c.pos.distance(d.route[d.next]) < 3. {
                d.next += 1;
            }
            if d.next >= d.route.len() {
                if d.incident {
                    c.speed = 0.;
                    d.parked = true;
                    arrived.push(d.car);
                    continue;
                }
                self.serial += 1;
                let target =
                    self.graph.nodes[(self.serial * 137 + d.car * 71) % self.graph.nodes.len()];
                if let Some((_, route)) = self.graph.route(c.pos, target) {
                    d.route = route;
                    d.next = 1;
                }
                continue;
            }
            let q = d.route[d.next];
            let dx = q.x - c.pos.x;
            let dz = q.z - c.pos.z;
            let error = (dx.atan2(dz) - c.angle)
                .sin()
                .atan2((dx.atan2(dz) - c.angle).cos());
            let mut desired = if d.incident { 21. } else { 14. };
            desired *= (1. - error.abs() / 2.).clamp(0.2, 1.);
            let yielding = snapshot.iter().enumerate().any(|(i, (p, heading))| {
                if i == d.car || (p.y - c.pos.y).abs() > 2. {
                    return false;
                }
                let x = p.x - c.pos.x;
                let z = p.z - c.pos.z;
                let ahead = x * c.angle.sin() + z * c.angle.cos();
                let across = x * c.angle.cos() - z * c.angle.sin();
                let crossing = (heading - c.angle).cos().abs() < 0.5;
                ahead > 0.
                    && ahead < 5.5 + c.speed.abs() * 0.7
                    && across.abs() < 2.8
                    && (!crossing || i < d.car)
            });
            let steer = (error * 5. * (1. + c.speed.abs() * 0.04)
                / (c.speed.abs().max(2.) * 0.22 * vehicles::CATALOG[c.kind].grip))
                .clamp(-1., 1.);
            let old = c.pos;
            vehicle_physics::drive(
                c,
                (
                    if yielding {
                        0.
                    } else {
                        ((desired - c.speed) * 0.45).clamp(-1., 1.)
                    },
                    steer,
                    yielding || c.speed > desired + 1.,
                ),
                dt,
                |x, z| world.surface(x, z),
                |p, r| {
                    blocked(p, r)
                        || terrain::railway::occupied(p, r, train_distance)
                        || snapshot.iter().enumerate().any(|(i, (q, _))| {
                            i != d.car && p.distance(*q) < r + 1.65 && (p.y - q.y).abs() < 2.
                        })
                },
            );
            if old.distance(c.pos) < 0.005 && !yielding {
                d.stuck += dt;
            } else {
                d.stuck = 0.;
            }
            // Replan on the road network rather than teleporting through an obstacle.
            if d.stuck > 4. {
                if let Some((_, route)) = self.graph.route(c.pos, *d.route.last().unwrap_or(&q)) {
                    d.route = route;
                    d.next = 0;
                }
                d.stuck = 0.;
            }
        }
        arrived
    }
}

pub(crate) fn prison_parking_zone(p: Vec3) -> bool {
    p.distance(Vec3::new(-116.0, 0.0, -110.0)) < 100.0
        || ((-180.0..=-20.0).contains(&p.x) && (-240.0..=-55.0).contains(&p.z))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn patrol_physically_reaches_store() {
        let w = World::new();
        let mut cars = Vec::new();
        let mut t = Traffic::with_seed(&w, &mut cars, 0);
        assert!(t.drivers.iter().filter(|d| d.police).count() >= 3);
        let target = Vec3::new(110., 0., 240.);
        let sent = t.dispatch(&cars, target, 1, None);
        assert_eq!(sent.len(), 1);
        let id = sent[0].1;
        for i in 0..9000 {
            let arrived = t.tick(
                &w,
                &mut cars,
                None,
                i as f32 / 60. * 11.2,
                1. / 60.,
                |p, r| w.blocked(p, r) || world::prison::barrier(p, r, false),
            );
            if arrived.contains(&id) {
                return;
            }
        }
        let d = t.drivers.iter().find(|d| d.car == id).unwrap();
        panic!(
            "car {:?} speed {} angle {} next {} {:?} route end {:?}",
            cars[id].pos,
            cars[id].speed,
            cars[id].angle,
            d.next,
            d.route.get(d.next),
            d.route.last()
        );
    }

    #[test]
    fn regenerated_traffic_uses_new_non_prison_spawn_points() {
        let w = World::new();
        let mut first = Vec::new();
        let mut second = Vec::new();
        let a = Traffic::with_seed(&w, &mut first, 1);
        let b = Traffic::with_seed(&w, &mut second, 2);
        assert!(a.drivers.iter().filter(|d| d.police).count() >= 3);
        assert!(b.drivers.iter().filter(|d| d.police).count() >= 3);
        assert!(first.iter().all(|c| !prison_parking_zone(c.pos)));
        assert!(second.iter().all(|c| !prison_parking_zone(c.pos)));
        assert_ne!(
            first.iter().map(|c| (c.pos.x, c.pos.z)).collect::<Vec<_>>(),
            second
                .iter()
                .map(|c| (c.pos.x, c.pos.z))
                .collect::<Vec<_>>()
        );
    }
}
