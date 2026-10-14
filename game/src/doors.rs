use buildings::Building;
use engine_core::Vec3;
pub struct Door {
    pub open: bool,
    pub angle: f32,
}
impl Door {
    pub fn new() -> Self {
        Self {
            open: false,
            angle: 0.,
        }
    }
    pub fn tick(&mut self, dt: f32) {
        let target = if self.open {
            std::f32::consts::FRAC_PI_2
        } else {
            0.
        };
        let d = (target - self.angle).clamp(-dt * 2.8, dt * 2.8);
        self.angle += d;
    }
    pub fn blocked(&self, b: &Building, p: Vec3, r: f32) -> bool {
        if !b.interior
            || p.y < b.p.y - 0.5
            || p.y > b.p.y + 4.5
            || p.distance(b.p) > b.size.x.max(b.size.z)
        {
            return false;
        }
        for side in [-1., 1.] {
            let a = self.angle * side;
            let hinge = Vec3::new(b.p.x + side * 1.5, b.p.y + 1.5, b.p.z - b.size.z / 2.);
            let cx = hinge.x - side * 0.75 * a.cos();
            let cz = hinge.z + side * 0.75 * a.sin();
            let dx = p.x - cx;
            let dz = p.z - cz;
            if (dx * a.cos() - dz * a.sin()).abs() < 0.75 + r
                && (dx * a.sin() + dz * a.cos()).abs() < 0.15 + r
            {
                return true;
            }
        }
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use buildings::Style;
    #[test]
    fn door_blocks_when_closed_and_clears_center_when_open() {
        let b = Building {
            id: 0,
            name: "test",
            p: Vec3::default(),
            size: Vec3::new(20., 5., 20.),
            style: Style::Civic,
            interior: true,
        };
        let p = Vec3::new(0., 1., -10.);
        let mut d = Door::new();
        assert!(d.blocked(&b, p, 0.45));
        d.open = true;
        d.tick(1.);
        assert!(!d.blocked(&b, p, 0.45));
    }
}
