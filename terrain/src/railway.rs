//! Shared, arc-length sampled alignment: the train and visible rails use identical poses.
use engine_core::Vec3;
use std::sync::OnceLock;
pub const SPACING: f32 = 10.8;
pub const COUNT: usize = 25;
const N: usize = 720;
struct Alignment {
    points: Vec<Vec3>,
    lengths: Vec<f32>,
}
fn alignment() -> &'static Alignment {
    static DATA: OnceLock<Alignment> = OnceLock::new();
    DATA.get_or_init(|| {
        let mut points: Vec<Vec3> = (0..N)
            .map(|i| {
                let a = i as f32 / N as f32 * std::f32::consts::TAU;
                let x = 930. * a.cos();
                let z = if a.sin() < 0. { 400. } else { 900. } * a.sin();
                Vec3::new(x, super::height(x, z).max(0.) + 3.8, z)
            })
            .collect();
        // A cyclic grade envelope raises approaches gradually instead of burying the line.
        for _ in 0..N {
            let mut changed = false;
            for dir in [false, true] {
                for k in 0..N {
                    let i = if dir { N - 1 - k } else { k };
                    let j = if dir { (i + 1) % N } else { (i + N - 1) % N };
                    let d = (points[i].x - points[j].x).hypot(points[i].z - points[j].z);
                    let h = points[j].y - d * 0.035;
                    if points[i].y + 0.0001 < h {
                        points[i].y = h;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        points.push(points[0]);
        let mut lengths = vec![0.];
        for p in points.windows(2) {
            lengths.push(lengths.last().copied().unwrap_or(0.) + p[0].distance(p[1]));
        }
        Alignment { points, lengths }
    })
}
pub fn length() -> f32 {
    *alignment().lengths.last().unwrap_or(&1.)
}
pub fn pose(distance: f32) -> (Vec3, f32) {
    let a = alignment();
    let d = distance.rem_euclid(length());
    let i = a
        .lengths
        .partition_point(|v| *v <= d)
        .saturating_sub(1)
        .min(N - 1);
    let p = a.points[i];
    let q = a.points[i + 1];
    let t = (d - a.lengths[i]) / (a.lengths[i + 1] - a.lengths[i]).max(0.001);
    (
        Vec3::new(
            p.x + (q.x - p.x) * t,
            p.y + (q.y - p.y) * t,
            p.z + (q.z - p.z) * t,
        ),
        (q.x - p.x).atan2(q.z - p.z),
    )
}
pub fn samples() -> &'static [Vec3] {
    &alignment().points
}
pub fn nearest(p: Vec3) -> (f32, Vec3) {
    let radius = if p.z < 0. { 400. } else { 900. };
    let a = (p.z / radius)
        .atan2(p.x / 930.)
        .rem_euclid(std::f32::consts::TAU);
    let center = (a / std::f32::consts::TAU * N as f32) as usize;
    let mut best = (f32::MAX, Vec3::default());
    for j in -3..=3 {
        let i = (center as isize + j).rem_euclid(N as isize) as usize;
        let a = alignment().points[i];
        let b = alignment().points[i + 1];
        let dx = b.x - a.x;
        let dz = b.z - a.z;
        let t =
            (((p.x - a.x) * dx + (p.z - a.z) * dz) / (dx * dx + dz * dz).max(0.001)).clamp(0., 1.);
        let q = Vec3::new(a.x + dx * t, a.y + (b.y - a.y) * t, a.z + dz * t);
        let d = (q.x - p.x).hypot(q.z - p.z);
        if d < best.0 {
            best = (d, q);
        }
    }
    best
}
pub fn carriage(clock: f32, wagon: usize) -> (Vec3, f32) {
    let (mut p, a) = pose(clock * 11.2 - wagon as f32 * SPACING);
    p.y += 0.8;
    (p, a)
}
pub fn locomotive(clock: f32) -> (Vec3, f32) {
    let (mut p, a) = pose(clock * 11.2 + SPACING);
    p.y += 0.8;
    (p, a)
}
pub fn occupied(p: Vec3, r: f32, clock: f32) -> bool {
    if nearest(p).0 > 7. + r {
        return false;
    }
    (0..=COUNT).any(|i| {
        let (q, a) = if i == COUNT {
            locomotive(clock)
        } else {
            carriage(clock, i)
        };
        let dx = p.x - q.x;
        let dz = p.z - q.z;
        let side = dx * a.cos() - dz * a.sin();
        let along = dx * a.sin() + dz * a.cos();
        side.abs() < 1.95 + r
            && along.abs() < 4.7 + r
            && p.y + 1.5 > q.y - 0.8
            && p.y - 0.8 < q.y + 3.8
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alignment_stays_above_land_and_grade_is_bounded() {
        for p in samples().windows(2) {
            assert!(p[0].y >= super::super::height(p[0].x, p[0].z) + 3.79);
            let d = (p[1].x - p[0].x).hypot(p[1].z - p[0].z);
            assert!((p[1].y - p[0].y).abs() <= d * 0.0351);
        }
    }
    #[test]
    fn cars_follow_rails_and_do_not_overlap_each_other() {
        for time in [0., 10., 80., 290., 1000.] {
            for i in 0..COUNT {
                let (p, _) = carriage(time, i);
                assert!((nearest(p).1.y + 0.8 - p.y).abs() < 0.01);
                if i > 0 {
                    assert!(p.distance(carriage(time, i - 1).0) > 10.5);
                }
            }
        }
    }
}
