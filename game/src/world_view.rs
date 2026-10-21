use engine_core::Vec3;
use renderer_gl21::{GpuMesh, Renderer, Texture};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use world::{cell, Streamer, World, CHUNK};
struct Resident {
    lod: u8,
    mesh: GpuMesh,
}
pub struct WorldView {
    world: Arc<World>,
    stream: Streamer,
    resident: HashMap<(i32, i32), Resident>,
    pending: HashSet<((i32, i32), u8)>,
    pub map: Texture,
    interior: Option<((usize, usize), GpuMesh)>,
    apartment: Option<(u64, GpuMesh)>,
    pub submitted_triangles: usize,
    pub interior_triangles: usize,
    roofs: HashMap<usize, GpuMesh>,
    pub visible: usize,
    pub bytes: usize,
    pub effective_distance: f32,
    visible_meshes: Vec<(f32, GpuMesh)>,
    pub occluded: usize,
}
impl WorldView {
    pub fn new(world: Arc<World>, r: &mut Renderer, p: Vec3, distance: f32, potato: bool) -> Self {
        let map = r.texture(256, 256, &world.navigation_image());
        let stream = Streamer::new(world.clone());
        let mut resident = HashMap::new();
        let center = cell(p.x, p.z);
        for dx in -1..=1 {
            for dz in -1..=1 {
                let key = (center.0 + dx, center.1 + dz);
                let d =
                    ((key.0 as f32 + 0.5) * CHUNK - p.x).hypot((key.1 as f32 + 0.5) * CHUNK - p.z);
                let lod = chunk_lod(distance, d, potato);
                let mesh = r.upload(&world.chunk(key, lod));
                resident.insert(key, Resident { lod, mesh });
            }
        }
        Self {
            world,
            stream,
            resident,
            pending: HashSet::new(),
            map,
            interior: None,
            apartment: None,
            submitted_triangles: 0,
            interior_triangles: 0,
            roofs: HashMap::new(),
            visible: 0,
            bytes: 0,
            effective_distance: distance,
            visible_meshes: Vec::with_capacity(256),
            occluded: 0,
        }
    }
    pub fn update(&mut self, r: &mut Renderer, p: Vec3, distance: f32, fps: f32, potato: bool) {
        let target = if fps > 0. && fps < 32. {
            distance * 0.8
        } else {
            distance
        };
        self.effective_distance += (target - self.effective_distance) * 0.005;
        self.effective_distance = self.effective_distance.clamp(128., distance.max(128.));
        let center = cell(p.x, p.z);
        let radius = (self.effective_distance / CHUNK).ceil() as i32 + 1;
        self.resident.retain(|key, value| {
            let keep =
                (key.0 - center.0).abs() <= radius + 1 && (key.1 - center.1).abs() <= radius + 1;
            if !keep {
                r.release(value.mesh);
            }
            keep
        });
        let start = std::time::Instant::now();
        for _ in 0..2 {
            let Some(data) = self.stream.poll() else {
                break;
            };
            self.pending.remove(&(data.key, data.lod));
            if (data.key.0 - center.0).abs() <= radius + 1
                && (data.key.1 - center.1).abs() <= radius + 1
            {
                let mesh = r.upload(&data.mesh);
                if let Some(old) = self.resident.insert(
                    data.key,
                    Resident {
                        lod: data.lod,
                        mesh,
                    },
                ) {
                    r.release(old.mesh);
                }
            }
            self.stream.recycle(data.mesh);
            if start.elapsed().as_millis() >= 3 {
                break;
            }
        }
        // Request near rings first; bounded channel prevents unbounded generation work.
        for ring in 0..=radius {
            for dx in -ring..=ring {
                for dz in -ring..=ring {
                    if dx.abs() != ring && dz.abs() != ring {
                        continue;
                    }
                    let key = (center.0 + dx, center.1 + dz);
                    if key.0 < -8 || key.0 >= 8 || key.1 < -8 || key.1 >= 8 {
                        continue;
                    }
                    let d = Vec3::new(
                        (key.0 as f32 + 0.5) * CHUNK,
                        0.,
                        (key.1 as f32 + 0.5) * CHUNK,
                    )
                    .distance(p);
                    if d > self.effective_distance + CHUNK {
                        continue;
                    }
                    let lod = chunk_lod(distance, d, potato);
                    if self.resident.get(&key).is_some_and(|c| c.lod == lod)
                        || self.pending.iter().any(|v| v.0 == key)
                    {
                        continue;
                    }
                    if self.stream.request(key, lod) {
                        self.pending.insert((key, lod));
                    }
                }
            }
        }
        let inside = interior_key(&self.world, p);
        if self.interior.as_ref().map(|i| i.0) != inside {
            if let Some((_, old)) = self.interior.take() {
                r.release(old);
            }
            if let Some((id, floor)) = inside {
                let mesh = if id == usize::MAX {
                    world::prison::sewer_mesh()
                } else {
                    self.world.buildings[id].interior_mesh_at(floor)
                };
                self.interior = Some(((id, floor), r.upload(&mesh)));
            }
        }
        // Interior cameras stay below the ceiling; keep the roof for enclosed rooms.
        for b in &self.world.buildings {
            if !b.interior {
                continue;
            }
            let wanted = b.p.distance(p) < self.effective_distance;
            if wanted && !self.roofs.contains_key(&b.id) {
                self.roofs.insert(b.id, r.upload(&b.roof_mesh()));
            } else if !wanted {
                if let Some(mesh) = self.roofs.remove(&b.id) {
                    r.release(mesh);
                }
            }
        }
        self.interior_triangles = self
            .interior
            .as_ref()
            .map_or(0, |(_, m)| m.vertices as usize / 3);
        self.bytes = self.resident.values().map(|v| v.mesh.bytes).sum::<usize>()
            + self.roofs.values().map(|v| v.bytes).sum::<usize>()
            + self.interior.as_ref().map_or(0, |i| i.1.bytes)
            + self.apartment.as_ref().map_or(0, |i| i.1.bytes);
    }
    pub fn set_active_apartment(&mut self, r: &mut Renderer, id: Option<u64>) {
        if self.apartment.as_ref().map(|instance| instance.0) == id {
            return;
        }
        if let Some((_, old)) = self.apartment.take() {
            r.release(old);
        }
        if let Some(id) = id {
            let mesh = buildings::apartment_rooms::instance_mesh(id);
            self.apartment = Some((id, r.upload(&mesh)));
        }
        self.bytes = self
            .resident
            .values()
            .map(|resident| resident.mesh.bytes)
            .sum::<usize>()
            + self.roofs.values().map(|mesh| mesh.bytes).sum::<usize>()
            + self
                .interior
                .as_ref()
                .map_or(0, |interior| interior.1.bytes)
            + self
                .apartment
                .as_ref()
                .map_or(0, |apartment| apartment.1.bytes);
    }
    pub fn draw(&mut self, r: &mut Renderer, eye: [f32; 3], yaw: f32, pitch: f32) {
        let triangles_before = r.triangles;
        r.fog(self.effective_distance);
        self.visible = 0;
        if let Some((_, mesh)) = self.apartment {
            r.mesh(mesh);
            self.visible = 1;
            self.submitted_triangles = r.triangles - triangles_before;
            return;
        }
        if self
            .interior
            .as_ref()
            .is_some_and(|((id, _), _)| *id == usize::MAX)
        {
            if let Some((_, mesh)) = self.interior {
                r.mesh(mesh);
            }
            self.submitted_triangles = r.triangles - triangles_before;
            return;
        }

        let aspect = r.width as f32 / r.height.max(1) as f32;
        self.visible_meshes.clear();
        self.occluded = 0;
        for (key, c) in &self.resident {
            let center = [
                (key.0 as f32 + 0.5) * CHUNK,
                35.,
                (key.1 as f32 + 0.5) * CHUNK,
            ];
            if !sphere_visible(
                center,
                245.,
                eye,
                yaw,
                pitch,
                aspect,
                self.effective_distance,
            ) {
                continue;
            }
            let distance = (center[0] - eye[0]).hypot(center[2] - eye[2]);
            let hidden = distance > 256.
                && self
                    .world
                    .buildings
                    .iter()
                    .filter(|b| !b.interior && b.p.distance(Vec3::new(eye[0], 0., eye[2])) < 100.)
                    .any(|b| {
                        let min = [b.p.x - b.size.x / 2., b.p.y, b.p.z - b.size.z / 2.];
                        let max = [
                            b.p.x + b.size.x / 2.,
                            b.p.y + b.size.y,
                            b.p.z + b.size.z / 2.,
                        ];
                        [-64., 64.].into_iter().all(|dx| {
                            [-64., 64.].into_iter().all(|dz| {
                                [-8., 245.].into_iter().all(|y| {
                                    occluded_ray(eye, [center[0] + dx, y, center[2] + dz], min, max)
                                })
                            })
                        })
                    });
            if hidden {
                self.occluded += 1;
                continue;
            }
            self.visible_meshes.push((distance, c.mesh));
        }
        self.visible_meshes
            .sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        for (_, mesh) in &self.visible_meshes {
            r.mesh(*mesh);
            self.visible += 1;
        }
        for (id, mesh) in &self.roofs {
            let b = &self.world.buildings[*id];
            let center = [b.p.x, b.p.y + b.size.y, b.p.z];
            let radius = b.size.x.hypot(b.size.z) * 0.5;
            if sphere_visible(
                center,
                radius,
                eye,
                yaw,
                pitch,
                aspect,
                self.effective_distance,
            ) {
                r.mesh(*mesh);
            }
        }
        if let Some((_, mesh)) = self.interior {
            r.mesh(mesh);
        }
        self.submitted_triangles = r.triangles - triangles_before;
    }
    pub fn active(&self) -> usize {
        self.resident.len()
    }
}
fn chunk_lod(distance: f32, d: f32, potato: bool) -> u8 {
    if potato {
        2
    } else if distance > 180. && d < distance * 0.5 {
        0
    } else if d < distance * 0.75 {
        1
    } else {
        2
    }
}
fn interior_key(world: &World, p: Vec3) -> Option<(usize, usize)> {
    if world::prison::underground(p) {
        return Some((usize::MAX, 0));
    }
    world
        .interior(p)
        .map(|id| (id, world.buildings[id].rendered_floor(p)))
}
pub(crate) fn sphere_visible(
    c: [f32; 3],
    radius: f32,
    eye: [f32; 3],
    yaw: f32,
    pitch: f32,
    aspect: f32,
    far: f32,
) -> bool {
    let dx = c[0] - eye[0];
    let dy = c[1] - eye[1];
    let dz = c[2] - eye[2];
    let right = dx * yaw.cos() + dz * yaw.sin();
    let horizontal = dx * yaw.sin() - dz * yaw.cos();
    let depth = horizontal * pitch.cos() - dy * pitch.sin();
    let up = dy * pitch.cos() + horizontal * pitch.sin();
    if depth + radius < 0. || depth - radius > far {
        return false;
    }
    let half_v = 0.7;
    let half_h = half_v * aspect;
    right.abs() <= depth * half_h + radius * (1. + half_h * half_h).sqrt()
        && up.abs() <= depth * half_v + radius * (1. + half_v * half_v).sqrt()
}

fn occluded_ray(eye: [f32; 3], target: [f32; 3], min: [f32; 3], max: [f32; 3]) -> bool {
    let mut near = 0f32;
    let mut far = 1f32;
    for i in 0..3 {
        let d = target[i] - eye[i];
        if d.abs() < 0.0001 {
            if eye[i] < min[i] || eye[i] > max[i] {
                return false;
            }
        } else {
            let a = (min[i] - eye[i]) / d;
            let b = (max[i] - eye[i]) / d;
            near = near.max(a.min(b));
            far = far.min(a.max(b));
            if near > far {
                return false;
            }
        }
    }
    near > 0. && far < 0.98
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opaque_occluder_requires_intersection() {
        assert!(occluded_ray(
            [0., 2., 0.],
            [0., 2., 100.],
            [-4., 0., 10.],
            [4., 8., 20.]
        ));
        assert!(!occluded_ray(
            [0., 2., 0.],
            [50., 2., 100.],
            [-4., 0., 10.],
            [4., 8., 20.]
        ));
        assert!(!occluded_ray(
            [0., 2., 0.],
            [0., 2., 5.],
            [-4., 0., 10.],
            [4., 8., 20.]
        ));
    }
    #[test]
    fn frustum_rejects_behind() {
        assert!(sphere_visible(
            [0., 0., -100.],
            5.,
            [0.; 3],
            0.,
            0.,
            1.7,
            400.
        ));
        assert!(!sphere_visible(
            [0., 0., 100.],
            5.,
            [0.; 3],
            0.,
            0.,
            1.7,
            400.
        ));
        assert!(!sphere_visible(
            [500., 0., -100.],
            5.,
            [0.; 3],
            0.,
            0.,
            1.7,
            400.
        ));
    }

    #[test]
    fn potato_quality_selects_flat_city_geometry() {
        assert_eq!(chunk_lod(128., 0., true), 2);
        assert_eq!(chunk_lod(180., 0., false), 1);
    }
}
#[cfg(test)]
mod interior_regressions {
    use super::*;
    #[test]
    fn low_boot_and_streaming_use_coarse_geometry() {
        assert_eq!(chunk_lod(180., 0., false), 1);
        assert_eq!(chunk_lod(180., 90., false), 1);
        assert_eq!(chunk_lod(450., 20., false), 0);
    }
    #[test]
    fn interior_cache_changes_with_floor_and_clears_outside() {
        let w = World::new();
        let b = w.buildings.iter().find(|b| b.name == "Office").unwrap();
        let mut p = b.p;
        p.y += 1.;
        assert_eq!(interior_key(&w, p), Some((b.id, 0)));
        p.y += 4.;
        assert_eq!(interior_key(&w, p), Some((b.id, 1)));
        assert_eq!(interior_key(&w, Vec3::new(310., 1., -20.)), None);
        assert_eq!(
            interior_key(&w, Vec3::new(-116., -3., -138.)),
            Some((usize::MAX, 0))
        );
    }
}
